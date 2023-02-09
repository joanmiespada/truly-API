use std::env;
use lib_users::models::user::User;
use lib_users::repositories::schema_user::create_schema_users;
use lib_users::services::users::{UsersService, UserManipulation };
use lib_users::services::login::LoginOps;
use lib_users::repositories::users::UsersRepo;
use lib_config::{Config, SECRETS_MANAGER_KEYS};
use spectral::{assert_that, result::ResultAssertions};
use testcontainers::*;
use aws_config::SdkConfig;
use aws_sdk_dynamodb::Credentials;
use aws_config::meta::region::RegionProviderChain;

async fn build_local_stack_connection(host_port: u16) -> SdkConfig {
    let endpoint_url = format!("http://127.0.0.1:{}", host_port);
    //let uri = Uri::from_str(&endpoint_uri).unwrap();
    //let endpoint_resolver = Endpoint::immutable_uri(uri);
    let region_provider = RegionProviderChain::default_provider().or_else("eu-central-1");
    let creds = Credentials::new(
        "test", 
        "test", 
        None, 
        None, 
        "test");

    let shared_config = aws_config::from_env()
        .region(region_provider)
        .endpoint_url(endpoint_url)
        //.endpoint_resolver(endpoint_resolver.unwrap())
        .credentials_provider(creds)
        .load()
        .await;

    //Client::new(&shared_config)
    return shared_config;
}

async fn create_secrets(
    client: &aws_sdk_secretsmanager::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    
    let secrets_json = r#"
    {
        "HMAC_SECRET" : "localtest_hmac_fgsdfg3rterfr2345weg@#$%WFRsdf",
        "JWT_TOKEN_BASE": "localtest_jwt_fdgsdfg@#$%Sdfgsdfg@#$3"
    }
    "#;
    
    client
        .create_secret()
        .name(SECRETS_MANAGER_KEYS.to_string())
        .secret_string(  secrets_json  )
        .send()
        .await?;

    Ok(())
}


#[tokio::test]
async fn login_user_device_test() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    env::set_var("RUST_LOG", "debug");
    env::set_var("ENVIRONMENT", "development");

    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();

    let mut local_stack = images::local_stack::LocalStack::default();
    local_stack.set_services("dynamodb");
    let node = docker.run(local_stack);
    let host_port = node.get_host_port_ipv4(4566);

    //create dynamodb tables against testcontainers.

    let shared_config = build_local_stack_connection(host_port).await;

    let dynamo_client = aws_sdk_dynamodb::Client::new(&shared_config);

    let creation1 = create_schema_users(&dynamo_client).await;
    assert_that(&creation1).is_ok();


    let mut config = Config::new();
    config.setup().await;
    config.set_aws_config(&shared_config); //rewrite configuration to use our current testcontainer instead

    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);

    let device = Some("1234".to_string());
    let device_ref = &device.to_owned();

    let mut new_user = User::new();
    new_user.set_device(&device.unwrap());
    

    let new_id = user_service.add_user(&mut new_user, &None).await?;


    let res = user_service.login(device_ref, &None, &None).await?;

    assert_eq!(new_id, res.user_id );

    Ok(())

}

#[tokio::test]
async fn login_user_email_password_test() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

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

    let secrets_client =aws_sdk_secretsmanager::Client::new(&shared_config);
    let creation2 = create_secrets(&secrets_client).await;
    assert_that(&creation2).is_ok();


    let mut config = Config::new();
    config.setup().await;
    config.set_aws_config(&shared_config); //rewrite configuration to use our current testcontainer instead
    config.load_secret(SECRETS_MANAGER_KEYS).await;

    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);

    let email = Some("pepe@test.cat.io".to_string());
    let email_ref = &email.to_owned();

    let password = Some("123456789aA$%^@2".to_string());
    let password_ref = &password.to_owned();

    let device = Some("device-1234".to_string());
    
    let mut new_user = User::new();
    new_user.set_email(&email.unwrap());
    new_user.set_device(&device.unwrap());
    

    let new_id = user_service.add_user(&mut new_user, password_ref).await?;


    let res = user_service.login(&None, email_ref, password_ref).await?;

    assert_eq!(new_id, res.user_id );

    Ok(())

}