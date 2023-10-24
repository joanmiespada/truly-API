use std::env;
use aws_sdk_dynamodb::Client;
use lib_config::config::Config;
use lib_config::environment::{DEV_ENV, ENV_VAR_ENVIRONMENT};
use lib_config::schema::Schema;
use lib_licenses::repositories::schema_subscription::SubscriptionSchema;
use lib_licenses::repositories::subscription::SubscriptionRepo;
use lib_licenses::services::subscription::SubscriptionService;
use lib_licenses::models::subscription::ConfirmedStatus;
use uuid::Uuid;
use spectral::prelude::*;
use testcontainers::*;

use lib_config::infra::build_local_stack_connection;

#[tokio::test]
async fn creation_subscription_table() {
    env::set_var("RUST_LOG", "debug");
    env::set_var("AWS_REGION", "eu-central-1");
    env::set_var(ENV_VAR_ENVIRONMENT, DEV_ENV);
    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config = build_local_stack_connection(host_port).await;

    let mut conf = Config::new();
    conf.setup().await;
    conf.set_aws_config(&shared_config);

    let creation = SubscriptionSchema::create_schema(&conf).await;
    assert_that(&creation).is_ok();

    let client = Client::new(&shared_config);
    let req = client.list_tables().limit(10);
    let list_tables_result = req.send().await.unwrap();

    assert_eq!(list_tables_result.table_names().unwrap().len(), 1);  
}

#[tokio::test]
async fn add_and_retrieve_subscription() {
    env::set_var("RUST_LOG", "debug");
    env::set_var("AWS_REGION", "eu-central-1");
    env::set_var(ENV_VAR_ENVIRONMENT, DEV_ENV);
    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);
    
    let shared_config = build_local_stack_connection(host_port).await;

    let mut conf = Config::new();
    conf.setup().await;
    conf.set_aws_config(&shared_config);
    
    let creation = SubscriptionSchema::create_schema(&conf).await;
    assert_that(&creation).is_ok();

    let repo = SubscriptionRepo::new(&conf);
    let service = SubscriptionService::new(repo);

    let user_id = "user123";
    let asset_id = Uuid::new_v4();
    let new_sub_id_op = service.intent(user_id.to_string(), asset_id).await;
    assert_that(&new_sub_id_op).is_ok();
    
    let new_sub_id_op2 = service.intent(user_id.to_string(), asset_id).await;
    assert_that(&new_sub_id_op2).is_ok();

    let new_sub_id = new_sub_id_op.unwrap();
    
    let retrieved_subscription = service.get(new_sub_id).await.unwrap().unwrap();
    assert_eq!(retrieved_subscription.user_id, user_id);
    assert_eq!(retrieved_subscription.asset_id, asset_id);
    assert_eq!(retrieved_subscription.confirmed, ConfirmedStatus::Disabled);
}

#[tokio::test]
async fn check_subscription_existence() {
    env::set_var("RUST_LOG", "debug");
    env::set_var("AWS_REGION", "eu-central-1");
    env::set_var(ENV_VAR_ENVIRONMENT, DEV_ENV);
    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config = build_local_stack_connection(host_port).await;

    let mut conf = Config::new();
    conf.setup().await;
    conf.set_aws_config(&shared_config);
    
    let creation = SubscriptionSchema::create_schema(&conf).await;
    assert_that(&creation).is_ok();

    let repo = SubscriptionRepo::new(&conf);
    let service = SubscriptionService::new(repo);

    let user_id = "user123";
    let asset_id = Uuid::new_v4();
    let subs_id = service.intent(user_id.to_string(), asset_id).await.unwrap();

    let exists = service.check_if_exists(user_id.to_string(), asset_id).await.unwrap();
    assert_eq!(exists.unwrap(), subs_id);
    
    let confirm_op = service.confirm(subs_id.clone()).await;
    assert_that!(confirm_op).is_ok();

    let confirm_op = service.confirm(subs_id.clone()).await;
    assert_that!(confirm_op).is_ok();

    let delete_op = service.delete(subs_id).await;
    assert_that!(delete_op).is_ok();

}

#[tokio::test]
async fn check_subscription_notify() {
    env::set_var("RUST_LOG", "debug");
    env::set_var("AWS_REGION", "eu-central-1");
    env::set_var(ENV_VAR_ENVIRONMENT, DEV_ENV);
    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config = build_local_stack_connection(host_port).await;

    let mut conf = Config::new();
    conf.setup().await;
    conf.set_aws_config(&shared_config);
    
    let creation = SubscriptionSchema::create_schema(&conf).await;
    assert_that(&creation).is_ok();

    let repo = SubscriptionRepo::new(&conf);
    let service = SubscriptionService::new(repo);

    let asset_id = Uuid::new_v4();

    let users = vec!["user1".to_string(), "user2".to_string(), "user3".to_string()];
    let mut intents = vec![];
    for user_id in users{
        let subs_id = service.intent(user_id.to_string(), asset_id).await.unwrap();
        intents.push(subs_id);
    }
    for intent in intents{
        service.confirm(intent).await.unwrap();
    }
    let _ = service.intent("user4".to_string(), asset_id).await;

    let search_op = service.find_asset_subscriptions(asset_id).await;
    assert_that!(search_op).is_ok();

    let subscriptions = search_op.unwrap(); 
    
    assert_eq!(subscriptions.len(), 3);

    let search_op2 = service.find_user_subscriptions("user1".to_string()).await;
    assert_that!(search_op2).is_ok();

    let subscriptions2 = search_op2.unwrap();
    assert_eq!(subscriptions2.len(), 1);

    let search_op3 = service.find_user_subscriptions("user4".to_string()).await;
    assert_that!(search_op3).is_ok();

    let subscriptions3 = search_op3.unwrap();
    assert_eq!(subscriptions3.len(), 0);



}