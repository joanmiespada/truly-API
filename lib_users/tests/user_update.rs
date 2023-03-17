use lib_config::infra::build_local_stack_connection;
use lib_config::{config::Config, secrets::SECRETS_MANAGER_APP_KEYS};
use lib_users::models::user::{User, UserStatus, UserRoles};
use lib_users::repositories::schema_user::create_schema_users;
use lib_users::repositories::users::UsersRepo;
use lib_users::services::login::LoginOps;
use lib_users::services::users::{UserManipulation, UsersService, UpdatableFildsUser};
use spectral::{assert_that, result::ResultAssertions};
use std::env;
use testcontainers::*;

async fn create_secrets(
    client: &aws_sdk_secretsmanager::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let secrets_json = r#"
    {
        "HMAC_SECRET" : "localtest_hmac_fgsdfg3rterfr2345weg@#$%WFRsdf",
        "JWT_TOKEN_BASE": "localtest_jwt_fdgsdfg@#$%Sdfgsdfg@#$3",
        "BLOCKCHAIN_GATEWAY_API_KEY": "sdgfh$#%^dfgh#$%^grdhf"
    }
    "#;

    client
        .create_secret()
        .name(SECRETS_MANAGER_APP_KEYS.to_string())
        .secret_string(secrets_json)
        .send()
        .await?;

    Ok(())
}

#[tokio::test]
async fn update_user_test() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env::set_var("RUST_LOG", "debug");
    env::set_var("ENVIRONMENT", "development");

    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();

    let mut local_stack = images::local_stack::LocalStack::default();
    local_stack.set_services("dynamodb,secretsmanager");
    let node = docker.run(local_stack);
    let host_port = node.get_host_port_ipv4(4566);

    //create dynamodb tables against testcontainers.

    let shared_config = build_local_stack_connection(host_port).await;

    let dynamo_client = aws_sdk_dynamodb::Client::new(&shared_config);
    let creation1 = create_schema_users(&dynamo_client).await;
    assert_that(&creation1).is_ok();

    let secrets_client = aws_sdk_secretsmanager::Client::new(&shared_config);
    let creation2 = create_secrets(&secrets_client).await;
    assert_that(&creation2).is_ok();

    let mut config = Config::new();
    config.setup().await;
    config.set_aws_config(&shared_config); //rewrite configuration to use our current testcontainer instead
    config.load_secret(SECRETS_MANAGER_APP_KEYS).await;

    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);

    let mut new_user = User::new();

    let device = Some("device-2r1234-12341234-12341234-1234-123".to_string());
    new_user.set_device(&device.clone().unwrap());

    let wallet = Some("wallet-2r1234-12341234-12341234-1234-123".to_string());
    new_user.set_wallet_address(&wallet.clone().unwrap());
    new_user.set_status( &UserStatus::Enabled );
    new_user.set_roles( &vec![UserRoles::Admin, UserRoles::Basic ]);
    let email = Some("pepe@test.cat.io".to_string());
    new_user.set_email(&email.unwrap());

    let password = Some("123456789aA$%^@2".to_string());

    let new_id = user_service.add(&mut new_user, &password).await?;

    let user_db = user_service.get_by_id(&new_id).await?;

    assert_eq!( user_db.creation_time(), new_user.creation_time());
    assert_eq!( user_db.status(), new_user.status());
    assert_eq!( *user_db.email().clone().unwrap(), *new_user.email().clone().unwrap());
    assert_eq!( *user_db.device().clone().unwrap(), *new_user.device().clone().unwrap());
    assert_eq!( *user_db.wallet_address().clone().unwrap(), *new_user.wallet_address().clone().unwrap());

    assert_eq!(new_id, *user_db.user_id());

    let updates = UpdatableFildsUser{
        email: Some("new_popo@test.io".to_string()),
        device: Some("new_device".to_string()),
        wallet: Some("new_wallet-124-123-123-123".to_string()),
        status: Some(  UserStatus::Disabled.to_string() )
    };

    let _ = user_service.update(user_db.user_id(), &updates).await;
    
    let user_db2 = user_service.get_by_id(&user_db.user_id()).await?;

    assert_eq!( user_db2.creation_time(), user_db.creation_time());
    assert_ne!( user_db2.status(), user_db.status());
    assert_ne!( user_db2.last_update_time(), user_db.last_update_time());
    assert_ne!( *user_db2.email().clone().unwrap(), *user_db.email().clone().unwrap());
    assert_ne!( *user_db2.device().clone().unwrap(), *user_db.device().clone().unwrap());
    assert_ne!( *user_db2.wallet_address().clone().unwrap(), *user_db.wallet_address().clone().unwrap());


    Ok(())
}

#[tokio::test]
async fn update_password_user_test() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env::set_var("RUST_LOG", "debug");
    env::set_var("ENVIRONMENT", "development");

    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();

    let mut local_stack = images::local_stack::LocalStack::default();
    local_stack.set_services("dynamodb,secretsmanager");
    let node = docker.run(local_stack);
    let host_port = node.get_host_port_ipv4(4566);

    //create dynamodb tables against testcontainers.

    let shared_config = build_local_stack_connection(host_port).await;

    let dynamo_client = aws_sdk_dynamodb::Client::new(&shared_config);
    let creation1 = create_schema_users(&dynamo_client).await;
    assert_that(&creation1).is_ok();

    let secrets_client = aws_sdk_secretsmanager::Client::new(&shared_config);
    let creation2 = create_secrets(&secrets_client).await;
    assert_that(&creation2).is_ok();

    let mut config = Config::new();
    config.setup().await;
    config.set_aws_config(&shared_config); //rewrite configuration to use our current testcontainer instead
    config.load_secret(SECRETS_MANAGER_APP_KEYS).await;

    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);

    let mut new_user = User::new();

    new_user.set_status( &UserStatus::Enabled );
    new_user.set_roles( &vec![UserRoles::Admin, UserRoles::Basic ]);
    let email = Some("pepe@test.cat.io".to_string());
    new_user.set_email(&email.clone().unwrap());
    let password = Some("123456789aA$%^@2".to_string());

    let new_id = user_service.add(&mut new_user, &password).await?;

    let res = user_service.login(&None, &None, &email, &password).await;
    assert_that(&res).is_ok();

    let new_password = Some("123456789aA$%^@2asdSDasd".to_string());
    user_service.update_password(&new_id, &new_password.clone().unwrap()).await?;

    let res2 = user_service.login(&None, &None, &email, &new_password).await;
    assert_that(&res2).is_ok();

    let res3 = user_service.login(&None, &None, &email, &password).await;
    assert_that(&res3).is_err();

    Ok(())
}