mod common;

use lib_config::environment::{DEV_ENV, ENV_VAR_ENVIRONMENT};
use lib_config::infra::build_local_stack_connection;
use lib_config::schema::Schema;
use lib_config::{config::Config, secrets::SECRETS_MANAGER_APP_KEYS};
use lib_users::models::user::User;
use lib_users::repositories::schema_user::UserAllSchema;
use lib_users::repositories::users::UsersRepo;
use lib_users::services::login::LoginOps;
use lib_users::services::users::{UserManipulation, UsersService};
use std::env;
use testcontainers::*;

use crate::common::create_secrets;


#[tokio::test]
async fn login_user_wallet_test() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env::set_var("RUST_LOG", "debug");
    env::set_var(ENV_VAR_ENVIRONMENT, DEV_ENV);
    env::set_var("AWS_REGION", "eu-central-1");

    let _ = env_logger::builder().is_test(true).try_init();

    let docker = clients::Cli::default();

    let mut local_stack = images::local_stack::LocalStack::default();
    local_stack.set_services("dynamodb,secretsmanager");
    let node = docker.run(local_stack);
    let host_port = node.get_host_port_ipv4(4566);

    //create dynamodb tables against testcontainers.

    let shared_config = build_local_stack_connection(host_port).await;

    //let dynamo_client = aws_sdk_dynamodb::Client::new(&shared_config);
    //let creation1 = create_schema_users(&dynamo_client).await;
    //assert_that(&creation1).is_ok();

    let secrets_client = aws_sdk_secretsmanager::Client::new(&shared_config);
    let creation2 = create_secrets(&secrets_client).await;
    assert!(creation2.is_ok());

    let mut config = Config::new();
    config.setup().await;
    config.set_aws_config(&shared_config); //rewrite configuration to use our current testcontainer instead
    config.load_secret(SECRETS_MANAGER_APP_KEYS.clone()).await;

    let creation = UserAllSchema::create_schema(&config).await;
    assert!(creation.is_ok());

    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);

    let wallet = Some("wallet-2r1234-12341234-12341234-1234-123".to_string());

    let mut new_user = User::new();
    new_user.set_wallet_address(&wallet.clone().unwrap());

    let new_id = user_service.add(&mut new_user, &None).await?;

    let res = user_service.login(&None, &wallet, &None, &None).await?;

    assert_eq!(new_id, res.user_id);

    Ok(())
}


