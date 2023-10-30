use std::env;

use aws_lambda_events::cloudwatch_events::CloudWatchEvent;
use chrono::Utc;
use http::{HeaderMap, HeaderValue};
use lambda_notifications::Notificator;
use lambda_notifications::collect_alerts;
use lambda_notifications::create_notifications;
//use lambda_notifications::function_handler;
use lambda_runtime::Context;
use lambda_runtime::LambdaEvent;
use lib_config::environment::{DEV_ENV, ENV_VAR_ENVIRONMENT};
use lib_config::infra::build_local_stack_connection;
use lib_config::result::ResultE;
use lib_config::schema::Schema;
use lib_engage::models::alert_similar::AlertSimilar;
use lib_engage::models::alert_similar::AlertSimilarBuilder;
use lib_engage::repositories::alert_similar::AlertSimilarRepo;
use lib_engage::repositories::schema_alert_similar::AlertSimilarSchema;
use lib_engage::repositories::schema_subscription::SubscriptionSchema;
use lib_engage::repositories::subscription::SubscriptionRepo;
use lib_engage::services::alert_similar::AlertSimilarService;
use lib_engage::services::subscription::SubscriptionService;
use lib_licenses::models::asset::Asset;
use lib_licenses::models::asset::SourceType;
use lib_licenses::repositories::assets::AssetRepo;
use lib_licenses::repositories::schema_asset::AssetAllSchema;
use lib_licenses::repositories::schema_owners::OwnerSchema;
use lib_licenses::repositories::shorter::ShorterRepo;
use lib_licenses::services::assets::AssetManipulation;
use lib_licenses::services::assets::AssetService;
use lib_licenses::services::assets::CreatableFildsAssetBuilder;
use lib_users::models::user::User;
use lib_users::models::user::UserRoles;
use lib_users::models::user::UserStatus;
use lib_users::repositories::schema_user::UserAllSchema;
use lib_users::repositories::users::UsersRepo;
use lib_users::services::users::UserManipulation;
use lib_users::services::users::UsersService;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use spectral::prelude::*;
use testcontainers::*;

fn _payload_lambda_event() -> LambdaEvent<CloudWatchEvent> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "lambda-runtime-aws-request-id",
        HeaderValue::from_static("my-id"),
    );
    headers.insert(
        "lambda-runtime-deadline-ms",
        HeaderValue::from_static("123"),
    );
    headers.insert(
        "lambda-runtime-invoked-function-arn",
        HeaderValue::from_static("arn::myarn"),
    );
    headers.insert(
        "lambda-runtime-trace-id",
        HeaderValue::from_static("arn::myarn"),
    );
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
    aux
}

fn generate_random_path(len: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect::<String>()
}

fn generate_random_url() -> String {
    let random_path = generate_random_path(10);
    let random_id = generate_random_path(5);
    format!("https://example.com/{}/{}", random_path, random_id)
}

fn generate_random_email() -> String {
    let local_part: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    let domain: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .map(char::from)
        .collect();
    format!("{}@{}.com", local_part, domain)
}

fn generate_random_password(len: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

async fn create_user(user_service: &UsersService) -> ResultE<User> {
    let mut new_user = User::new();

    new_user.set_status(&UserStatus::Enabled);
    new_user.set_roles(&vec![UserRoles::Basic]);
    let email = Some(generate_random_email());
    new_user.set_email(&email.unwrap());

    //let password = Some(generate_random_password(10));

    let _new_id = user_service.add(&mut new_user, &None).await?;

    Ok(new_user)
}

async fn create_asset(asset_service: &AssetService, user: &Option<User>) -> ResultE<Asset> {
    let aux = CreatableFildsAssetBuilder::default()
        .url(generate_random_url())
        .license(None)
        .hash(None)
        .hash_algorithm(None)
        .latitude(None)
        .longitude(None)
        .father(None)
        .source_details(None)
        .source(SourceType::Others)
        .build()?;

    let us = match user {
        Some(u) => Some(u.user_id().clone()),
        None => None,
    };

    let id = asset_service.add(&aux, &us).await?;

    let asset = asset_service.get_by_id(&id).await?;

    Ok(asset)
}

async fn create_subscription(
    subscription_service: &SubscriptionService<SubscriptionRepo>,
    user: &User,
    asset: &Asset,
) -> ResultE<()> {
    let id = subscription_service
        .intent(user.user_id().clone(), asset.id().clone())
        .await?;
    subscription_service.confirm(id).await?;
    Ok(())
}

async fn create_alert(
    alert_service: &AlertSimilarService<AlertSimilarRepo>,
    asset1: &Asset,
    asset2: &Asset,
) -> ResultE<AlertSimilar> {
    let mut aux = AlertSimilarBuilder::default();
    aux.source_type(Some("Hashes".to_string()));
    aux.origin_asset_id(Some(asset1.id().clone()));
    aux.origin_frame_id(Some(uuid::Uuid::new_v4()));
    aux.origin_frame_second(Some(1.0));
    aux.origin_frame_url(Some("/testOrigin/testOrigin".to_string()));
    aux.origin_hash_type(Some("Hash.Phash".to_string()));
    aux.origin_hash_id(Some(uuid::Uuid::new_v4()));

    aux.similar_asset_id(Some(asset2.id().clone()));
    aux.similar_frame_id(Some(uuid::Uuid::new_v4()));
    aux.similar_frame_second(Some(10.0));
    aux.similar_frame_url(Some("/testSimilar/testSimilar".to_string()));

    let simil = alert_service.add(&mut aux).await?;
    Ok(simil)
}

#[tokio::test]
async fn check_asset_alerts_notifications() -> ResultE<()> {
    env::set_var("RUST_LOG", "debug");
    env::set_var("AWS_REGION", "eu-central-1");
    env::set_var(ENV_VAR_ENVIRONMENT, DEV_ENV);
    env::set_var("PAGINATION_TOKEN_ENCODER", "asdfghjkl");

    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config = build_local_stack_connection(host_port).await;

    let mut config = lib_config::config::Config::new();
    config.setup().await;
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

    let user1 = create_user(&user_service).await?;
    let user2 = create_user(&user_service).await?;
    let user3 = create_user(&user_service).await?;
    let _user4 = create_user(&user_service).await?;

    let asset1 = create_asset(&asset_service, &None).await?;
    let asset2 = create_asset(&asset_service, &None).await?;
    let asset3 = create_asset(&asset_service, &None).await?;
    let asset4 = create_asset(&asset_service, &None).await?;

    create_subscription(&subscription_service, &user1, &asset1).await?;
    create_subscription(&subscription_service, &user2, &asset1).await?;
    create_subscription(&subscription_service, &user3, &asset1).await?;

    create_subscription(&subscription_service, &user2, &asset2).await?;
    create_subscription(&subscription_service, &user3, &asset2).await?;

    create_subscription(&subscription_service, &user3, &asset3).await?;

    create_alert(&alert_service, &asset1, &asset2).await?;
    create_alert(&alert_service, &asset4, &asset2).await?;

    let alerts = collect_alerts(&alert_service, Some(1)).await?;

    assert_eq!(alerts.len(), 2);

    let notifis: Notificator = create_notifications(&alerts, &subscription_service, &user_service, &asset_service).await?;

    assert_eq!(notifis.len(), 3);
    
    Ok(())
}
