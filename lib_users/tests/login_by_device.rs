use lib_config::config::Config;
use lib_config::environment::{DEV_ENV, ENV_VAR_ENVIRONMENT};
use lib_config::infra::build_local_stack_connection;
use lib_config::schema::Schema;
use lib_users::models::user::User;
use lib_users::repositories::schema_user::UserAllSchema;
use lib_users::repositories::users::UsersRepo;
use lib_users::services::login::LoginOps;
use lib_users::services::users::{UserManipulation, UsersService};
use spectral::{assert_that, result::ResultAssertions};
use std::env;
use testcontainers::*;

#[tokio::test]
async fn login_user_device_test() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env::set_var("RUST_LOG", "debug");
    env::set_var(ENV_VAR_ENVIRONMENT, DEV_ENV);

    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();

    let mut local_stack = images::local_stack::LocalStack::default();
    local_stack.set_services("dynamodb");
    let node = docker.run(local_stack);
    let host_port = node.get_host_port_ipv4(4566);

    let shared_config = build_local_stack_connection(host_port).await;

    let mut config = Config::new();
    config.setup().await;
    config.set_aws_config(&shared_config); //rewrite configuration to use our current testcontainer instead

    let creation = UserAllSchema::create_schema(&config).await;
    assert_that(&creation).is_ok();

    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);

    let device = "1234".to_string();

    let mut new_user = User::new();
    new_user.set_device(&device);

    let new_id = user_service.add(&mut new_user, &None).await?;

    let res = user_service
        .login(&Some(device), &None, &None, &None)
        .await?;

    assert_eq!(new_id, res.user_id);

    Ok(())
}
