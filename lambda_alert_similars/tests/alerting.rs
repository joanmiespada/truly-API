use std::env;
use lib_config::environment::{DEV_ENV, ENV_VAR_ENVIRONMENT};
use lib_config::infra::build_local_stack_connection;
use lib_config::result::ResultE;
use lib_config::schema::Schema;
use lib_engage::repositories::alert_similar::AlertSimilarRepo;
use lib_engage::repositories::schema_alert_similar::AlertSimilarSchema;
use lib_engage::repositories::sender::SMTP_TEST_SERVER;
use lib_engage::services::alert_similar::AlertSimilarService;
use lib_hash_objs::similar_alert::AlertExternalPayload;
use testcontainers::*;
use lambda_alert_similars::my_lambda::similar_found::store_similar_found_successfully;

#[tokio::test]
async fn create_alert_test() -> ResultE<()> {
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
    local_stack.set_services("dynamodb");
    let node = docker.run(local_stack);
    let host_port = node.get_host_port_ipv4(4566);

    let shared_config = build_local_stack_connection(host_port).await;

    let mut config = lib_config::config::Config::new();
    config.setup().await;
    config.set_aws_config(&shared_config);

    let creation = AlertSimilarSchema::create_schema(&config).await;
    assert!(creation.is_ok());

    let notification_repo = AlertSimilarRepo::new(&config);
    let notification_service = AlertSimilarService::new(notification_repo);

    let data = AlertExternalPayload {
        source_type: Some("hashes".to_string()),
        origin_hash_id: Some(uuid::Uuid::new_v4() ),
        origin_hash_type: Some("chash".to_string()),
        origin_frame_id: Some(uuid::Uuid::new_v4() ),
        origin_frame_second: Some(8.0),
        origin_frame_url: Some("3e9da557-3ff6-4087-9add-bb9363e03755/t000000008_mirror.png".to_string()),
        origin_asset_id: Some(uuid::Uuid::new_v4() ),
        similar_frame_id: Some(uuid::Uuid::new_v4() ),
        similar_frame_second: Some(3.0),
        similar_frame_url: Some("becdfd7b-1fac-4fee-9254-ed0b7b9110f5/t000000003_mirror.png".to_string()),
        similar_asset_id: Some(uuid::Uuid::new_v4() )
    };

    let aux = store_similar_found_successfully(
        &data,
        &config,
        &notification_service,
    )
    .await;

    assert!(aux.is_ok());

    Ok(())
}
