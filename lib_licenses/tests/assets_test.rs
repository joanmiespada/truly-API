use std::collections::HashMap;
use std::env;
use std::str::FromStr;

use aws_sdk_dynamodb::Client;
use lib_config::config::Config;
use lib_config::environment::{DEV_ENV, ENV_VAR_ENVIRONMENT};
use lib_config::schema::Schema;
use lib_licenses::models::asset::{AssetStatus, SourceType};
use lib_licenses::repositories::assets::AssetRepo;
use lib_licenses::repositories::schema_asset::AssetAllSchema;
use lib_licenses::repositories::schema_owners::OwnerSchema;
use lib_licenses::repositories::shorter::ShorterRepo;
use lib_licenses::services::assets::{
    AssetManipulation, AssetService, CreatableFildsAsset, UpdatableFildsAsset,
};
use spectral::prelude::*;
use testcontainers::*;
use url::Url;

use lib_config::infra::build_local_stack_connection;
use uuid::Uuid;

#[tokio::test]
async fn creation_table() {
    env::set_var("RUST_LOG", "debug");
    env::set_var(ENV_VAR_ENVIRONMENT, DEV_ENV);
    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config = build_local_stack_connection(host_port).await;

    let mut conf = Config::new();
    conf.setup().await;
    conf.set_aws_config(&shared_config);

    let creation = AssetAllSchema::create_schema(&conf).await;
    assert_that(&creation).is_ok();
    let creation = OwnerSchema::create_schema(&conf).await;
    assert_that(&creation).is_ok();

    let client = Client::new(&shared_config);

    let req = client.list_tables().limit(10);
    let list_tables_result = req.send().await.unwrap();

    assert_eq!(list_tables_result.table_names().unwrap().len(), 4);
}

#[tokio::test]
async fn add_assets() {
    env::set_var("RUST_LOG", "debug");
    env::set_var(ENV_VAR_ENVIRONMENT, DEV_ENV);
    env::set_var("AWS_REGION", "eu-central-1");
    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();
    //let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let node = docker.run(images::local_stack::LocalStack::default());
    let host_port = node.get_host_port_ipv4(4566);
    env::set_var("AWS_ENDPOINT", format!("http://127.0.0.1:{}",host_port));


    //let shared_config = build_local_stack_connection(host_port).await;

    let mut conf = Config::new();
    conf.setup().await;
    //conf.set_aws_config(&shared_config);

    let creation = AssetAllSchema::create_schema(&conf).await;
    assert_that(&creation).is_ok();
    let creation = OwnerSchema::create_schema(&conf).await;
    assert_that(&creation).is_ok();

    let repo_assets = AssetRepo::new(&conf);
    let repo_shorters = ShorterRepo::new(&conf);
    let service = AssetService::new(repo_assets, repo_shorters);

    let as1 = CreatableFildsAsset {
        url: "http://www.file1.com/test1.mp4".to_string(),
        hash: None, // Some("hash1234".to_string()),
        license: None, //Some("gnu".to_string()),
        hash_algorithm: None, // Some("MD5".to_string()),
        longitude: None,
        latitude: None,
        father: None,
        source: SourceType::Others,
        source_details: None,
    };

    let user = String::from_str("user1").unwrap();

    let new_op = service.add(&as1, &user).await;
    assert_that!(&new_op).is_ok();
    let new_id = new_op.unwrap();

    let get_op = service.get_by_id(&new_id).await;
    assert_that!(&get_op).is_ok();

    let aass11 = get_op.unwrap();
    let url = aass11.url().clone().unwrap();
    assert_eq!(url.to_string(), as1.url);

    let up_as = UpdatableFildsAsset {
        license: Some("mit".to_string()),
        status: Some("Disabled".to_string()),
    };

    let upd_op = service.update(aass11.id(), &up_as).await;
    assert_that!(&upd_op).is_ok();

    let get2_op = service.get_by_id(&new_id).await;
    assert_that!(&get2_op).is_ok();
    let ass3 = get2_op.unwrap();
    assert_eq!(*ass3.state(), AssetStatus::Disabled);
}

fn list_of_assets() -> HashMap<String, Vec<Url>> {
    let mut aux = HashMap::new();

    aux.insert(
        "user1".to_string(),
        vec![
            Url::parse("http://1.com/sdf1.png").unwrap(),
            Url::parse("http://2.com/sdf2.png").unwrap(),
        ],
    );

    aux.insert(
        "user2".to_string(),
        vec![
            Url::parse("http://3.com/sdf3.png").unwrap(),
            Url::parse("http://4.com/sdf4.png").unwrap(),
            Url::parse("http://5.com/sdf5.png").unwrap(),
        ],
    );
    aux.insert(
        "user3".to_string(),
        vec![Url::parse("http://6.com/sdf6.png").unwrap()],
    );

    aux.insert("user4".to_string(), vec![]);

    return aux;
}

fn list_of_assets_tree() -> (HashMap<String, (Vec<Url>, Option<Uuid>)>, Vec<Uuid>) {
    let mut aux = HashMap::new();

    let asset1 = Uuid::new_v4();
    let asset2 = Uuid::new_v4();
    let asset3 = Uuid::new_v4();
    let asset4 = Uuid::new_v4();

    let master = vec![asset1, asset2, asset3, asset4];

    aux.insert(
        "user1".to_string(),
        (
            vec![
                Url::parse("http://1.com/sdf1.png").unwrap(),
                Url::parse("http://2.com/sdf2.png").unwrap(),
            ],
            Some(asset1),
        ),
    );

    aux.insert(
        "user2".to_string(),
        (
            vec![
                Url::parse("http://3.com/sdf3.png").unwrap(),
                Url::parse("http://4.com/sdf4.png").unwrap(),
                Url::parse("http://5.com/sdf5.png").unwrap(),
            ],
            Some(asset2),
        ),
    );
    aux.insert(
        "user3".to_string(),
        (vec![Url::parse("http://6.com/sdf6.png").unwrap()], None),
    );

    aux.insert("user4".to_string(), (vec![], None));

    return (aux, master);
}

#[tokio::test]
async fn check_ownership() {
    env::set_var("RUST_LOG", "debug");
    env::set_var("RUST_BACKTRACE", "full");

    env_logger::builder().is_test(true).init();

    //let _ = pretty_env_logger::try_init();
    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config = build_local_stack_connection(host_port).await;

    // let mut creation = create_schema_assets_all(&client).await;
    // assert_that(&creation).is_ok();
    // creation = create_schema_owners(&client).await;
    // assert_that(&creation).is_ok();

    let mut conf = lib_config::config::Config::new();
    conf.set_aws_config(&shared_config);

    let creation = AssetAllSchema::create_schema(&conf).await;
    assert_that(&creation).is_ok();
    let creation = OwnerSchema::create_schema(&conf).await;
    assert_that(&creation).is_ok();

    let repo_assets = AssetRepo::new(&conf);
    let repo_shorters = ShorterRepo::new(&conf);
    let service = AssetService::new(repo_assets, repo_shorters);

    let payload = list_of_assets();
    let mut list_of_ids = HashMap::new();
    for user in payload {
        for ass in user.1 {
            let username = user.0.clone();

            let mut as1 = CreatableFildsAsset {
                url: ass.to_string(),
                hash: Some("hash1234".to_string()),
                hash_algorithm: Some("MD5".to_string()),
                license: Some(String::from_str("gnu").unwrap()),
                longitude: None,
                latitude: None,
                father: None,
                source: SourceType::Others,
                source_details: None,
            };

            let new_op = service.add(&mut as1, &username).await;
            assert_that!(&new_op).is_ok();

            let new_id = new_op.unwrap().clone();
            println!(
                "added user: {} with asset: {}",
                username,
                new_id.to_string()
            );
            list_of_ids.insert(new_id, username);
        }
    }

    let mut total = service.get_all(0, 100).await.unwrap();
    assert_eq!(total.len(), 6);
    for doc in total {
        println!("id: {}", doc.id().to_string())
    }

    total = service.get_by_user_id(&"user1".to_string()).await.unwrap();
    assert_eq!(total.len(), 2);

    total = service.get_by_user_id(&"user2".to_string()).await.unwrap();
    assert_eq!(total.len(), 3);

    let mut test1212 = list_of_ids.iter().next().unwrap();
    let asset1 = service.get_by_user_asset_id(test1212.0, test1212.1).await;

    assert_that(&asset1).is_ok();

    test1212 = list_of_ids.iter().next().unwrap();
    let ass = service.get_by_id(test1212.0).await;
    assert_that(&ass).is_ok();
}

#[tokio::test]
async fn check_asset_tree_father_son() {
    //let _ = pretty_env_logger::try_init();
    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config = build_local_stack_connection(host_port).await;

    // let mut creation = create_schema_assets_all(&client).await;
    // assert_that(&creation).is_ok();
    // creation = create_schema_owners(&client).await;
    // assert_that(&creation).is_ok();

    let mut conf = lib_config::config::Config::new();
    conf.set_aws_config(&shared_config);

    let creation = AssetAllSchema::create_schema(&conf).await;
    assert_that(&creation).is_ok();
    let creation = OwnerSchema::create_schema(&conf).await;
    assert_that(&creation).is_ok();

    let repo_assets = AssetRepo::new(&conf);
    let repo_shorters = ShorterRepo::new(&conf);
    let service = AssetService::new(repo_assets, repo_shorters);

    let payload = list_of_assets_tree();
    let mut list_of_ids = HashMap::new();
    for user in payload.0 {
        for ass in user.1 .0 {
            let username = user.0.clone();

            let mut as1 = CreatableFildsAsset {
                url: ass.to_string(),
                hash: Some("hash1234".to_string()),
                hash_algorithm: Some("MD5".to_string()),
                license: Some(String::from_str("gnu").unwrap()),
                longitude: None,
                latitude: None,
                father: user.1 .1,
                source: SourceType::Others,
                source_details: None,
            };

            let new_op = service.add(&mut as1, &username).await;
            assert_that!(&new_op).is_ok();

            let new_id = new_op.unwrap().clone();
            let father = match user.1 .1 {
                None => "no father".to_string(),
                Some(id) => id.to_string(),
            };
            println!(
                "added user: {} with asset: {} and father id: {}",
                username,
                new_id.to_string(),
                father
            );
            list_of_ids.insert(new_id, user.1 .1);
        }
    }

    for doc in list_of_ids {
        let asset1 = service.get_by_id(&doc.0).await.unwrap();
        assert_eq!(*asset1.father(), doc.1);
    }
}
