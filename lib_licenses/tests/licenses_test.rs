use aws_sdk_dynamodb::Client;
use chrono::Utc;
use lib_config::infra::build_local_stack_connection;
use lib_licenses::models::license::{License, LicenseStatus, Royalty};
use lib_licenses::repositories::licenses::LicenseRepo;
use lib_licenses::repositories::schema_licenses::create_schema_licenses;
use lib_licenses::services::licenses::{LicenseManipulation, LicenseService};
use rand::seq::SliceRandom;
use rand::Rng;
use spectral::prelude::*;
use std::env;
use testcontainers::*;
use uuid::Uuid;

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
async fn add_licenses() {
    env::set_var("RUST_LOG", "debug");
    env::set_var("ENVIRONMENT", "development");

    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config = build_local_stack_connection(host_port).await;
    let client = Client::new(&shared_config);

    let creation = create_schema_licenses(&client).await;

    assert_that(&creation).is_ok();

    let mut conf = lib_config::config::Config::new();
    conf.set_aws_config(&shared_config);

    let repo = LicenseRepo::new(&conf);
    let service = LicenseService::new(repo);
    let asset_id = Some(Uuid::new_v4());
    let mut licenses = vec![
        generate_random_license(None),
        generate_random_license(asset_id),
        generate_random_license(asset_id),
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

    let search_op = service.get_by_asset(&asset_id.unwrap()).await;
    assert_that!(&search_op).is_ok();
    assert_eq!(search_op.unwrap().len(), 2);

    let search_op2 = service.get_by_license(licenses.first().unwrap().id()).await;
    assert_that!(&search_op2).is_ok();
    assert_eq!(search_op2.unwrap().unwrap(), *licenses.first().unwrap());



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
