use std::env;
use lambda_license::my_lambda::subscribe::subscribe::create_intent;
use lib_async_ops::sns::create;
use lib_config::environment::{DEV_ENV, ENV_VAR_ENVIRONMENT};
use lib_config::infra::build_local_stack_connection;
use lib_config::result::ResultE;
use lib_config::schema::Schema;
use lib_engage::repositories::schema_subscription::SubscriptionSchema;
use lib_engage::repositories::sender::SenderEmailsRepo;
use lib_engage::repositories::sender::SMTP_TEST_SERVER;
use lib_engage::repositories::subscription::SubscriptionRepo;
use lib_engage::services::subscription::SubscriptionService;
use lib_licenses::repositories::assets::AssetRepo;
use lib_licenses::repositories::schema_asset::AssetAllSchema;
use lib_licenses::repositories::schema_owners::OwnerSchema;
use lib_licenses::repositories::shorter::ShorterRepo;
use lib_licenses::services::assets::AssetService;
use lib_licenses::services::video::VideoService;
use lib_users::repositories::schema_user::UserAllSchema;
use lib_users::repositories::users::UsersRepo;
use lib_users::services::users::UsersService;
use lib_util_jwt::randoms::{generate_random_email, generate_random_url};
use serde_json::json;
use testcontainers::*;
use lambda_http::{Request, Body, http::HeaderValue, http::Method, Context };


fn mock_request(email: String, url:String) -> Request {
    let payload = json!({
        "email": email,
        "url": url
    }).to_string();

    let mut req = Request::new(Body::from(payload));
    *req.method_mut() = Method::POST;
    req.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));
    req
}

// Mock the Context
fn mock_context() -> Context {
    Context::default()
}


#[tokio::test]
async fn create_intent_test() -> ResultE<()> {
    env::set_var("RUST_LOG", "debug");
    env::set_var("AWS_REGION", "eu-central-1");
    env::set_var(ENV_VAR_ENVIRONMENT, DEV_ENV);
    env::set_var("PAGINATION_TOKEN_ENCODER", "asdfghjkl");
    env::set_var("DEFAULT_PAGE_SIZE", "25");
    env::set_var("SMTP_HOST", SMTP_TEST_SERVER);
    env::set_var("SMTP_USER", "test");
    env::set_var("SMTP_PASSW", "test");

    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();
    let mut local_stack = images::local_stack::LocalStack::default();
    local_stack.set_services("dynamodb,sns");
    let node = docker.run(local_stack);
    let host_port = node.get_host_port_ipv4(4566);

    let shared_config = build_local_stack_connection(host_port).await;

    let mut config = lib_config::config::Config::new();
    config.setup().await;
    config.set_aws_config(&shared_config);

    let topic_arn = create(&config, "hashes_similar_video_in_topic".to_string()).await?;
    env::set_var("HASHES_SIMILAR_VIDEO_IN_TOPIC", topic_arn);
    config.refresh_env_vars();

    let creation = AssetAllSchema::create_schema(&config).await;
    assert!(creation.is_ok());
    let creation = OwnerSchema::create_schema(&config).await;
    assert!(creation.is_ok());
    let creation = SubscriptionSchema::create_schema(&config).await;
    assert!(creation.is_ok());
    let creation = UserAllSchema::create_schema(&config).await;
    assert!(creation.is_ok());

    let subscription_repo = SubscriptionRepo::new(&config);
    let send_repo = SenderEmailsRepo::new(&config);
    let subscription_service = SubscriptionService::new(subscription_repo, send_repo);

    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);

    let asset_repo = AssetRepo::new(&config);
    let shorter_repo = ShorterRepo::new(&config);
    let asset_service = AssetService::new(asset_repo, shorter_repo);
    
    let video_service = VideoService::new(asset_service.to_owned(), config.to_owned());

    let email = generate_random_email();

    let req = mock_request(email.to_owned(), generate_random_url());
    let c = mock_context();

    let aux = create_intent(
        &req,
        &c,
        &config,
        &asset_service,
        &video_service,
        &user_service,
        &subscription_service,
    )
    .await;

    assert!(aux.is_ok());


    let req = mock_request(email, generate_random_url());

    let aux = create_intent(
        &req,
        &c,
        &config,
        &asset_service,
        &video_service,
        &user_service,
        &subscription_service,
    )
    .await;

    assert!(aux.is_ok());



    Ok(())
}
