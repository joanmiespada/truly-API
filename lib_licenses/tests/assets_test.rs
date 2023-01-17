use std::str::FromStr;

use lib_licenses::{repositories::schema_asset::create_schema_assets, models::asset::Asset};  
use lib_licenses::repositories::assets::AssetRepo;
use lib_licenses::services::assets::{AssetService, AssetManipulation};
use aws_sdk_dynamodb::{Client };
use spectral::prelude::*;
use testcontainers::*;
use url::Url;

use crate::common::build_dynamodb;



#[tokio::test]
async fn creation_table() {
    //let _ = pretty_env_logger::try_init();
    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config= build_dynamodb(host_port).await;

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
    conf.set_aws_config( &shared_config); 

    let repo = AssetRepo::new(&conf);
    let service = AssetService::new(repo);

    let url1: Url =  Url::parse("http://www.file1.com/test1.mp4").unwrap();
    let hash1: String = "hash1234".to_string();
    let lic1: String = String::from_str("lic1").unwrap();


    let mut as1 = Asset::new();
    as1.set_url( &Some(url1) );
    as1.set_hash(&Some(hash1));
    as1.set_license(&Some(lic1));

    let new_op = service.add(&mut as1).await;

    assert_that!(&new_op).is_ok();
    
}