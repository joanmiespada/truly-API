use std::env;
use lib_config::{config::Config, infra::{build_local_stack_connection, create_key, create_secret_manager_keys, create_secret_manager_secret_key, store_secret_key, restore_secret_key}};
use testcontainers::*;

#[tokio::test]
async fn set_up_secret() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "debug");
    env::set_var("ENVIRONMENT", "development");

    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();

    let mut local_stack = images::local_stack::LocalStack::default();
    local_stack.set_services("secretsmanager,kms");
    let node = docker.run(local_stack);
    let host_port = node.get_host_port_ipv4(4566);

    let shared_config = build_local_stack_connection(host_port).await;

    let mut config = Config::new();
    config.setup().await;
    config.set_aws_config(&shared_config);

    let keys_client = aws_sdk_kms::client::Client::new(&shared_config);
    let kms_id = create_key(&keys_client).await?;
    let secrets_client = aws_sdk_secretsmanager::client::Client::new(&shared_config);

    let secrets_json = r#"
    {
        "HMAC_SECRET" : "localtest_hmac_1234RGsdfg#$%",
        "JWT_TOKEN_BASE": "localtest_jwt_sd543ERGds235$%^"
    }
    "#;
    create_secret_manager_keys(secrets_json, &secrets_client).await?;
    create_secret_manager_secret_key(&secrets_client).await?;

    let secret: &str = "4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d"; // secret key example
    
    store_secret_key(secret, kms_id.as_str(), &config).await?;
    let res = restore_secret_key(kms_id.as_str(), &config).await?;

    assert_eq!(secret, res);

    Ok(())
}
