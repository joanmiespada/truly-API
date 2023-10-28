use std::env;

use aws_lambda_events::cloudwatch_events::CloudWatchEvent;
use chrono::Utc;
use lambda_runtime::Context;
use lambda_runtime::LambdaEvent;
use lib_config::environment::{DEV_ENV, ENV_VAR_ENVIRONMENT};
use lib_config::infra::build_local_stack_connection;
use lib_config::schema::Schema;
use lib_engage::repositories::alert_similar::AlertSimilarRepo;
use lib_engage::repositories::schema_alert_similar::AlertSimilarSchema;
use lib_engage::repositories::schema_subscription::SubscriptionSchema;
use lib_engage::repositories::subscription::SubscriptionRepo;
use lib_engage::services::alert_similar::AlertSimilarService;
use lib_engage::services::subscription::SubscriptionService;
use lib_licenses::repositories::assets::AssetRepo;
use lib_licenses::repositories::schema_asset::AssetAllSchema;
use lib_licenses::repositories::schema_owners::OwnerSchema;
use lib_licenses::repositories::shorter::ShorterRepo;
use lib_licenses::services::assets::AssetService;
use lib_users::repositories::schema_user::UserAllSchema;
use lib_users::repositories::users::UsersRepo;
use lib_users::services::users::UsersService;
use spectral::prelude::*;
use testcontainers::*;
use http::{HeaderMap, HeaderValue};
use lambda_notifications::function_handler;

#[tokio::test]
async fn check_asset_tree_father_son() {
    env::set_var("RUST_LOG", "debug");
    env::set_var(ENV_VAR_ENVIRONMENT, DEV_ENV);

    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config = build_local_stack_connection(host_port).await;

    let mut config = lib_config::config::Config::new();
    config.set_aws_config(&shared_config);

    let creation = AssetAllSchema::create_schema(&config).await;
    assert_that(&creation).is_ok();
    let creation = OwnerSchema::create_schema(&config).await;
    assert_that(&creation).is_ok();
    let creation = SubscriptionSchema::create_schema(&config).await;
    assert_that(&creation).is_ok();
    let creation = UserAllSchema::create_schema(&config).await;
    assert_that(&creation).is_ok();
    let creation = AlertSimilarSchema::create_schema(&config).await;
    assert_that(&creation).is_ok();

    let alert_repo = AlertSimilarRepo::new(&config);
    let alert_service = AlertSimilarService::new(alert_repo);

    let subscription_repo = SubscriptionRepo::new(&config);
    let subscription_service = SubscriptionService::new(subscription_repo);

    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);

    let asset_repo = AssetRepo::new(&config);
    let shorter_repo = ShorterRepo::new(&config);
    let asset_service = AssetService::new(asset_repo, shorter_repo);


    let mut headers = HeaderMap::new();
    headers.insert("lambda-runtime-aws-request-id", HeaderValue::from_static("my-id"));
    headers.insert("lambda-runtime-deadline-ms", HeaderValue::from_static("123"));
    headers.insert(
        "lambda-runtime-invoked-function-arn",
        HeaderValue::from_static("arn::myarn"),
    );
    headers.insert("lambda-runtime-trace-id", HeaderValue::from_static("arn::myarn"));
    let tried = Context::try_from(headers);

    let aux: LambdaEvent<CloudWatchEvent> = LambdaEvent::new(
        CloudWatchEvent {
            version: None,
            id: None,
            detail_type: None,
            source: None,
            //account: None,
            time: Utc::now(),
            region: None,
            resources: Vec::new(),
            detail: None,
            account_id: None,
        },
        tried.unwrap(),
    );

    let res = function_handler(
        aux,
        &config,
        &alert_service,
        &subscription_service,
        &user_service,
        &asset_service,
    )
    .await;
    assert_that(&res).is_ok();
}
