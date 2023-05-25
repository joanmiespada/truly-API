use aws_sdk_dynamodb::Client;
use chrono::Utc;
use lib_config::infra::build_local_stack_connection;
use lib_licenses::models::asset::Asset;
use lib_licenses::models::license::{License, LicenseStatus, Royalty};
use lib_licenses::repositories::assets::{AssetRepo, AssetRepository};
use lib_licenses::repositories::licenses::LicenseRepo;
use lib_licenses::repositories::schema_asset::create_schema_assets_all;
use lib_licenses::repositories::schema_licenses::create_schema_licenses;
use lib_licenses::repositories::schema_owners::create_schema_owners;
use lib_licenses::services::licenses::{LicenseManipulation, LicenseService};
use rand::seq::SliceRandom;
use rand::Rng;
use spectral::prelude::*;
use url::Url;
use std::env;
use testcontainers::*;
use uuid::Uuid;

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[tokio::test]
async fn creation_table() {
    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config = build_local_stack_connection(host_port).await;

    let client = Client::new(&shared_config);

    let creation = create_schema_licenses(&client).await;

    assert_that(&creation).is_ok();

    let req = client.list_tables().limit(10);
    let list_tables_result = req.send().await.unwrap();

    assert_eq!(list_tables_result.table_names().unwrap().len(), 1);
}

#[tokio::test]
async fn run_licenses()-> ResultE<()> {
    env::set_var("RUST_LOG", "debug");
    env::set_var("ENVIRONMENT", "development");

    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config = build_local_stack_connection(host_port).await;
    let client = Client::new(&shared_config);

    create_schema_owners(&client).await?;
    create_schema_assets_all(&client).await?;
    let creation = create_schema_licenses(&client).await;

    assert_that(&creation).is_ok();

    let mut conf = lib_config::config::Config::new();
    conf.set_aws_config(&shared_config);

    let repo = LicenseRepo::new(&conf);
    let ass_repo = AssetRepo::new(&conf);

    let user_id = Uuid::new_v4().to_string();
    let asset_id1;
    let asset_id2;
    
    let mut asset = Asset::new2( Uuid::new_v4(),  Url::parse("http://a.xyz")? ,  "hash1234".to_string());
    asset_id1 = ass_repo.add(&asset, &user_id).await?;

    asset = Asset::new2( Uuid::new_v4(), Url::parse("http://b.xyz")? ,  "hash5678".to_string());
    asset_id2 = ass_repo.add(&asset, &user_id).await?;

    let service = LicenseService::new(repo,ass_repo);

    let mut licenses = vec![
        generate_random_license(Some(asset_id1)),
        generate_random_license(Some(asset_id2)),
        generate_random_license(Some(asset_id2)),
    ];
    let total_len = licenses.len();
    for mut license in licenses.iter_mut() {
        let new_op = service.create(&mut license).await;
        assert_that!(&new_op).is_ok();
    }

    let res_op = service.get_all(0, 10).await;
    assert_that!(&res_op).is_ok();
    let res = res_op.unwrap();
    assert_eq!(res.len(), total_len);

    for license in licenses.iter() {
        let new_op = service.get_by_id(license.id(), license.asset_id()).await;

        assert_that!(&new_op).is_ok();
        let lic = new_op.ok().unwrap().unwrap();
        assert_eq!(lic, *license)
    }

    let search_op = service.get_by_asset(&asset_id2).await;
    assert_that!(&search_op).is_ok();
    assert_eq!(search_op.unwrap().len(), 2);

    let search_op2 = service.get_by_license(licenses.first().unwrap().id()).await;
    assert_that!(&search_op2).is_ok();
    assert_eq!(search_op2.unwrap().unwrap(), *licenses.first().unwrap());

    let target = licenses.first().unwrap().clone();
    let search_op3 = service.delete(&target).await;
    assert_that!(&search_op3).is_ok();
    let after_del_op = service.get_all(0, 10).await;
    let after_del = after_del_op .unwrap();
    assert_eq!(after_del.len(), total_len-1);

    Ok(())


}

fn generate_random_license(asset_id: Option<Uuid>) -> License {
    let mut rng = rand::thread_rng();

    let mut license = License::new();

    license.set_id(Uuid::nil());
    license.set_creation_time(Utc::now());
    license.set_last_update_time(Utc::now());
    match asset_id{
        None =>  license.set_asset_id(Uuid::new_v4()),
        Some(ass) => license.set_asset_id(ass)
    }
    license.set_version(rng.gen_range(1..=10));

    license.set_right_to_free_distribute(rng.gen::<bool>());
    license.set_if_you_distribute_mention_me(rng.gen::<bool>());
    license.set_right_to_modify(rng.gen::<bool>());
    license.set_if_you_modify_mention_me(rng.gen::<bool>());
    license.set_right_to_use_broadcast_media(rng.gen::<bool>());
    license.set_right_to_use_press_media(rng.gen::<bool>());

    let rights = vec![
        generate_random_royalty(),
        generate_random_royalty(),
        generate_random_royalty(),
    ];

    license.set_rights(rights);
    license.set_status(LicenseStatus::Enabled);

    license
}

fn generate_random_royalty() -> Royalty {
    let mut rng = rand::thread_rng();

    let price = rng.gen_range(0.0..=1000.0);
    let location = generate_random_string();

    Royalty { price, location }
}

fn generate_random_string() -> String {
    let mut rng = rand::thread_rng();
    let length: usize = rng.gen_range(5..=10);
    let letters: Vec<char> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
    let random_string: String = (0..length)
        .map(|_| *letters.choose(&mut rng).unwrap())
        .collect();
    random_string
}
