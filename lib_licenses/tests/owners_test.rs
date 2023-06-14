use std::env;

use aws_sdk_dynamodb::Client;
use lib_config::environment::{DEV_ENV, ENV_VAR_ENVIRONMENT};
use lib_config::infra::build_local_stack_connection;
use lib_config::schema::Schema;
use lib_licenses::models::owner::Owner;
use lib_licenses::repositories::owners::OwnerRepo;
use lib_licenses::repositories::schema_owners::OwnerSchema;
use lib_licenses::services::owners::{OwnerManipulation, OwnerService};
use spectral::prelude::*;
use testcontainers::*;
use uuid::Uuid;

#[tokio::test]
async fn creation_table() {
    env::set_var("RUST_LOG", "debug");
    env::set_var(ENV_VAR_ENVIRONMENT, DEV_ENV);
    env_logger::builder().is_test(true).init();
    //let _ = pretty_env_logger::try_init();
    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config = build_local_stack_connection(host_port).await;

    let mut conf = lib_config::config::Config::new();
    conf.setup().await;
    conf.set_aws_config(&shared_config);

    let creation = OwnerSchema::create_schema(&conf).await;
    assert_that(&creation).is_ok();

    let client = Client::new(&shared_config);
    let req = client.list_tables().limit(10);
    let list_tables_result = req.send().await.unwrap();

    assert_eq!(list_tables_result.table_names().unwrap().len(), 1);
}

#[tokio::test]
async fn add_owners() {
    env::set_var("RUST_LOG", "debug");
    env::set_var(ENV_VAR_ENVIRONMENT, DEV_ENV);

    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config = build_local_stack_connection(host_port).await;
    //let client = Client::new(&shared_config);

    let mut conf = lib_config::config::Config::new();
    conf.setup().await;
    conf.set_aws_config(&shared_config);

    let creation = OwnerSchema::create_schema(&conf).await;
    assert_that(&creation).is_ok();

    let repo = OwnerRepo::new(&conf);
    let service = OwnerService::new(repo);

    let asset1 = Uuid::new_v4();
    let asset2 = Uuid::new_v4();
    let asset3 = Uuid::new_v4();

    let items = vec![("user1", asset1), ("user1", asset2), ("user2", asset3)];

    let mut as1 = Owner::new();
    for item in items {
        as1.set_user_id(&item.0.to_string());
        as1.set_asset_id(&item.1);

        let new_op = service.add(&mut as1).await;

        assert_that!(&new_op).is_ok();
    }
    let mut res_op = service.get_by_user(&"user1".to_string()).await;
    assert_that!(&res_op).is_ok();
    let mut res = res_op.unwrap();
    assert_eq!(res.len(), 2);
    res_op = service.get_by_user(&"user2".to_string()).await;
    assert_that!(&res_op).is_ok();
    res = res_op.unwrap();
    assert_eq!(res.len(), 1);

    let mut res_op2 = service.get_by_asset(&asset1).await;
    assert_that!(&res_op2).is_ok();
    let mut res2 = res_op2.unwrap();
    assert_eq!("user1", res2.user_id());

    res_op2 = service.get_by_asset(&asset3).await;
    assert_that!(&res_op2).is_ok();
    res2 = res_op2.unwrap();
    assert_eq!("user2", res2.user_id());
}
