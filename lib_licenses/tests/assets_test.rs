use std::collections::HashMap;
use std::str::FromStr;

use aws_sdk_dynamodb::Client;
use lib_licenses::repositories::assets::AssetRepo;
use lib_licenses::repositories::schema_owners::create_schema_owners;
use lib_licenses::services::assets::{AssetManipulation, AssetService};
use lib_licenses::{models::asset::Asset, repositories::schema_asset::create_schema_assets};
use spectral::prelude::*;
use testcontainers::*;
use url::Url;

use crate::build_dynamodb;

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



fn list_of_assets() -> HashMap<String, Vec<Url>> {
    let mut aux = HashMap::new();

    aux.insert(
        "user1".to_string(),
        vec![
                Url::parse("http://1.com/sdf1.png").unwrap(),
                Url::parse("http://2.com/sdf2.png").unwrap()
        ],
    );

    aux.insert(
        "user2".to_string(),
        vec![
                Url::parse("http://3.com/sdf3.png").unwrap(),
                Url::parse("http://4.com/sdf4.png").unwrap(),
                Url::parse("http://5.com/sdf5.png").unwrap()
        ],
    );
    aux.insert(
        "user3".to_string(),
        vec![
                Url::parse("http://6.com/sdf6.png").unwrap()
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
    let mut list_of_ids = HashMap::new();
    for user in payload {
        for ass in user.1 {
            let username = user.0.clone();
            let url1: Url = ass;
            let hash1: String = "hash1234".to_string();
            let lic1: String = String::from_str("lic1").unwrap();

            let mut as1 = Asset::new();
            as1.set_url(&Some(url1));
            as1.set_hash(&Some(hash1));
            as1.set_license(&Some(lic1));

            let new_op = service.add(&mut as1, &username).await;
            assert_that!(&new_op).is_ok();

            let new_id = new_op.unwrap().clone();
            println!("added user: {} with asset: {}", username, new_id.to_string());
            list_of_ids.insert(new_id, username);

        }
    }
  
    let mut total = service.get_all(0, 100).await.unwrap();
    assert_eq!(total.len(), 6);
    for doc in total {
        println!("id: {}",doc.id().to_string())
    }

    total = service.get_by_user_id(&"user1".to_string()).await.unwrap();
    assert_eq!(total.len(), 2);

    total = service.get_by_user_id(&"user2".to_string()).await.unwrap();
    assert_eq!(total.len(), 3);
    
    let mut test1212 = list_of_ids.iter().next().unwrap();
    let asset1 = service
        .get_by_user_asset_id(
            test1212.0,
            test1212.1
     ).await;
     
    assert_that(&asset1).is_ok();

    test1212 = list_of_ids.iter().next().unwrap();
    let ass = service.get_by_id(test1212.0).await;
    assert_that(&ass).is_ok();



}
