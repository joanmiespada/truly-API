
use lib_licenses::models::owner::Owner;
use lib_licenses::repositories::schema_owners::create_schema_owners;
use lib_licenses::repositories::owners::OwnerRepo;
use lib_licenses::services::owners::{OwnerService, OwnerManipulation};
use aws_sdk_dynamodb::{Client };
use spectral::prelude::*;
use testcontainers::*;
use uuid::Uuid;

use crate::build_dynamodb;



#[tokio::test]
async fn creation_table() {
    //let _ = pretty_env_logger::try_init();
    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config= build_dynamodb(host_port).await;

    let client = Client::new(&shared_config);
    
    let creation = create_schema_owners(&client).await;
    
    assert_that(&creation).is_ok();

    let req = client.list_tables().limit(10);
    let list_tables_result = req.send().await.unwrap();

    assert_eq!(list_tables_result.table_names().unwrap().len(), 1);
}

#[tokio::test]
async fn add_owners() {
    
    //let _ = pretty_env_logger::try_init();
    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config = build_dynamodb(host_port).await;
    let client = Client::new(&shared_config);
    
    let creation = create_schema_owners(&client).await;
    
    assert_that(&creation).is_ok();

    let mut conf = lib_config::Config::new();
    conf.set_aws_config( &shared_config); 

    let repo = OwnerRepo::new(&conf);
    let service = OwnerService::new(repo);

    let mut as1 = Owner::new();
    as1.set_user_id( &"hello1user1".to_string());
    as1.set_asset_id( &Uuid::new_v4() );

    let new_op = service.add(&mut as1).await;

    assert_that!(&new_op).is_ok();
    
}