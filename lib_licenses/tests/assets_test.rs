use std::collections::HashMap;
use std::env;
use std::str::FromStr;

use aws_sdk_dynamodb::Client;
use lib_licenses::repositories::assets::AssetRepo;
use lib_licenses::repositories::schema_owners::create_schema_owners;
use lib_licenses::services::assets::{AssetManipulation, AssetService};
use lib_licenses::{models::asset::Asset, repositories::schema_asset::create_schema_assets};
use spectral::prelude::*;
use testcontainers::*;
use url::Url;
use uuid::Uuid;

use crate::common::build_dynamodb;

#[tokio::test]
async fn creation_table() {
    //let _ = pretty_env_logger::try_init();
    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config = build_dynamodb(host_port).await;

    let client = Client::new(&shared_config);

    let creation = create_schema_assets(&client).await;

    assert_that(&creation).is_ok();

    let req = client.list_tables().limit(10);
    let list_tables_result = req.send().await.unwrap();

    assert_eq!(list_tables_result.table_names().unwrap().len(), 1);
}

#[tokio::test]
async fn add_assets() {
    //let _ = pretty_env_logger::try_init();
    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config = build_dynamodb(host_port).await;
    let client = Client::new(&shared_config);

    let creation = create_schema_assets(&client).await;

    assert_that(&creation).is_ok();

    let mut conf = lib_config::Config::new();
    conf.set_aws_config(&shared_config);

    let repo = AssetRepo::new(&conf);
    let service = AssetService::new(repo);

    let url1: Url = Url::parse("http://www.file1.com/test1.mp4").unwrap();
    let hash1: String = "hash1234".to_string();
    let lic1: String = String::from_str("lic1").unwrap();

    let mut as1 = Asset::new();
    as1.set_url(&Some(url1));
    as1.set_hash(&Some(hash1));
    as1.set_license(&Some(lic1));

    let user = String::from_str("user1").unwrap();

    let new_op = service.add(&mut as1, &user).await;

    assert_that!(&new_op).is_ok();
}

fn list_of_assets() -> HashMap<String, Vec<String>> {
    let mut aux = HashMap::new();

    aux.insert(
        "user1".to_string(),
        vec![
            "7b3834b6-267f-4fa4-ae65-a632d7701689".to_string(), 
            "deb0fc6c-cd94-43b2-9b04-93bdb3bc28cf".to_string(),
        ],
    );

    aux.insert(
        "user2".to_string(),
        vec![
            "b1c5dbfd-a727-402d-b301-e07890ef914a".to_string(),
            "da5589ea-adc7-4ccb-ab30-342e0ed20395".to_string(),
            "62381466-013d-4893-b083-d7ee180ecbb8".to_string(),
        ],
    );
    aux.insert(
        "user3".to_string(),
        vec![
            "f6e11b16-48a6-4e01-8439-509320cce02e".to_string(), 
        ],
    );

    aux.insert("user4".to_string(), vec![]);

    return aux;
}
#[tokio::test]
async fn check_ownership() {
    //let _ = pretty_env_logger::try_init();
    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config = build_dynamodb(host_port).await;
    let client = Client::new(&shared_config);

    let mut creation = create_schema_assets(&client).await;
    assert_that(&creation).is_ok();
    creation = create_schema_owners(&client).await;
    assert_that(&creation).is_ok();


    let mut conf = lib_config::Config::new();
    conf.set_aws_config(&shared_config);

    let repo = AssetRepo::new(&conf);
    let service = AssetService::new(repo);

    let payload = list_of_assets();

    for user in payload {
        for ass in user.1 {
            let url1: Url = Url::parse("http://www.file1.com/test1.mp4").unwrap();
            let hash1: String = "hash1234".to_string();
            let lic1: String = String::from_str("lic1").unwrap();

            let mut as1 = Asset::new();
            //as1.set_id(&Uuid::from_str(ass.as_str()).unwrap());
            as1.set_url(&Some(url1));
            as1.set_hash(&Some(hash1));
            as1.set_license(&Some(lic1));

            println!("adding user: {} with asset: {}", user.0, ass);
            let new_op = service.add(&mut as1, &user.0).await;

            assert_that!(&new_op).is_ok();
        }
    }
 
    let mut total = service.get_all(0, 100).await.unwrap();
    assert_eq!(total.len(), 6);

    total = service.get_by_user_id(&"user1".to_string()).await.unwrap();
    assert_eq!(total.len(), 2);

    total = service.get_by_user_id(&"user2".to_string()).await.unwrap();
    assert_eq!(total.len(), 3);

    let asset_id1 = Uuid::from_str(&"f6e11b16-48a6-4e01-8439-509320cce02e").unwrap();
    let user1 = String::from_str(&"user3").unwrap();
    let asset1 = service
        .get_by_user_asset_id(
            &asset_id1,
            &user1
     ).await.unwrap();
     assert_eq!(asset1.id().clone(), asset_id1 );




}
