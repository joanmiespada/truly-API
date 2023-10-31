use std::env;
use std::str::FromStr;
use aws_sdk_dynamodb::Client;
use lib_config::config::Config;
use lib_config::environment::{DEV_ENV, ENV_VAR_ENVIRONMENT};
use lib_config::result::ResultE;
use lib_config::schema::Schema;
use lib_engage::repositories::schema_subscription::SubscriptionSchema;
use lib_engage::repositories::sender::{SenderEmailsRepo, SMTP_TEST_SERVER};
use lib_engage::repositories::subscription::SubscriptionRepo;
use lib_engage::services::subscription::SubscriptionService;
use lib_engage::models::subscription::ConfirmedStatus;
use lib_licenses::models::asset::AssetBuilder;
use lib_users::models::user::UserBuilder;
use url::Url;
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
async fn add_and_retrieve_subscription() -> ResultE<()> {
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
    let repo_emails = SenderEmailsRepo::new(&conf);
    let service = SubscriptionService::new(repo,repo_emails);

    let user_id = "user123";
    let user = UserBuilder::default().user_id(user_id.to_string()).build()?;

    let asset_id = Uuid::new_v4();
    let asset = AssetBuilder::new().id(asset_id).build();

    let new_sub_id_op = service.intent(user.clone(), asset.clone()).await;
    assert_that(&new_sub_id_op).is_ok();
    
    let new_sub_id_op2 = service.intent(user, asset).await;
    assert_that(&new_sub_id_op2).is_ok();

    let new_sub_id = new_sub_id_op.unwrap();
    
    let retrieved_subscription = service.get(new_sub_id).await.unwrap().unwrap();
    assert_eq!(retrieved_subscription.user_id, user_id);
    assert_eq!(retrieved_subscription.asset_id, asset_id);
    assert_eq!(retrieved_subscription.confirmed, ConfirmedStatus::Disabled);
    Ok(())  
}

#[tokio::test]
async fn check_subscription_existence() -> ResultE<()> {
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
    let repo_emails = SenderEmailsRepo::new(&conf);
    let service = SubscriptionService::new(repo, repo_emails);

    let user_id = "user123";
    let user = UserBuilder::default().user_id(user_id.to_string()).build()?;

    let asset_id = Uuid::new_v4();
    let asset = AssetBuilder::new().id(asset_id).build();
    
    let subs_id = service.intent(user, asset).await.unwrap();

    let exists = service.check_if_exists(user_id.to_string(), asset_id).await.unwrap();
    assert_eq!(exists.unwrap(), subs_id);
    
    let confirm_op = service.confirm(subs_id.clone()).await;
    assert_that!(confirm_op).is_ok();

    let confirm_op = service.confirm(subs_id.clone()).await;
    assert_that!(confirm_op).is_ok();

    let delete_op = service.delete(subs_id).await;
    assert_that!(delete_op).is_ok();
    Ok(())

}

#[tokio::test]
async fn check_subscription_notify() ->ResultE<()> {
    env::set_var("RUST_LOG", "debug");
    env::set_var("AWS_REGION", "eu-central-1");
    env::set_var(ENV_VAR_ENVIRONMENT, DEV_ENV);
    env::set_var("SMTP_HOST", SMTP_TEST_SERVER);
    env::set_var("SMTP_USER", "test");
    env::set_var("SMTP_PASSW", "test");

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
    let repo_emails = SenderEmailsRepo::new(&conf);
    let service = SubscriptionService::new(repo, repo_emails);


    let user_names = vec!["user1".to_string(), "user2".to_string(), "user3".to_string()];
    let users: Result<Vec<_>, _> = user_names
        .into_iter()
        .map(|user_id| UserBuilder::default()
                                .user_id(user_id.clone())
                                .email(Some(format!("{}@example.com",user_id)))
                                .build()
            )
        .collect();
    let users = users?;


    let asset_id = Uuid::new_v4();
    let asset = AssetBuilder::new().id(asset_id).url( Url::from_str("http://test1.test")? ) .build();


    let mut intents = vec![];
    for user in users{
        let subs_id = service.intent(user, asset.clone()).await.unwrap();
        intents.push(subs_id);
    }
    for intent in intents{
        service.confirm(intent).await.unwrap();
    }
    let user4= UserBuilder::default().user_id("user4".to_string()).email(Some("user4@example.com".to_string())).build()?;
    let _ = service.intent(user4, asset).await;

    let search_op = service.find_users_subscribed_to(asset_id).await;
    assert_that!(search_op).is_ok();

    let subscriptions = search_op.unwrap(); 
    
    assert_eq!(subscriptions.len(), 3);

    let search_op2 = service.find_assets_subscribed_to("user1".to_string()).await;
    assert_that!(search_op2).is_ok();

    let subscriptions2 = search_op2.unwrap();
    assert_eq!(subscriptions2.len(), 1);

    let search_op3 = service.find_assets_subscribed_to("user4".to_string()).await;
    assert_that!(search_op3).is_ok();

    let subscriptions3 = search_op3.unwrap();
    assert_eq!(subscriptions3.len(), 0);

    Ok(())

}