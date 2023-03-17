use lib_config::config::Config;
use lib_config::infra::{
    create_key, create_secret_manager_keys,
    create_secret_manager_secret_key, store_secret_key,
};
use lib_licenses::repositories::{schema_asset, schema_keypairs, schema_owners};
use lib_licenses::services::contract::deploy_contract_locally;
use lib_users::models::user::User;
use lib_users::repositories::schema_user;
use lib_users::repositories::users::UsersRepo;
use lib_users::services::users::PromoteUser;
use lib_users::services::users::{UserManipulation, UsersService};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Blockchain {
    blockchain_url: Url,
    contract_owner: String,
    contract_address: String,
}
pub async fn non_terraformed_dependencies(
    environment: String,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let secrets_json;
    if environment == "development" {
        secrets_json = include_str!("../res/secrets_development.json");
    } else {
        secrets_json = include_str!("../res/secrets_prod_stage.json");
    }
    let secret_key_raw;
    if environment == "development" {
        secret_key_raw = include_str!("../res/key_development.txt");
    } else {
        secret_key_raw = include_str!("../res/key_prod_stage.txt");
    }
    let aux_user;
    if environment == "development" {
        aux_user = include_str!("../res/adminuser_development.json");
    } else {
        aux_user = include_str!("../res/adminuser_prod_stage.json");
    }
    let blockchain;
    if environment == "development" {
        blockchain = include_str!("../res/blockchain_development.json")
    } else {
        blockchain = include_str!("../res/blockchain_prod_stage.json")
    }

    let client = aws_sdk_dynamodb::Client::new(config.aws_config());
    schema_owners::create_schema_owners(&client).await?;
    schema_asset::create_schema_assets(&client).await?;
    schema_keypairs::create_schema_keypairs(&client).await?;
    schema_user::create_schema_users(&client).await?;
    drop(client);
    let client_key = aws_sdk_kms::client::Client::new(config.aws_config());
    let key_id = create_key(&client_key).await?;
    drop(client_key);
    let client_sec = aws_sdk_secretsmanager::client::Client::new(config.aws_config());
    create_secret_manager_keys(secrets_json, &client_sec).await?;
    create_secret_manager_secret_key(&client_sec).await?;
    store_secret_key(&secret_key_raw, &key_id, &config).await?;
    drop(client_sec);

    let blockchain_json: Blockchain = serde_json::from_str(blockchain).unwrap();
    let blockchain_contract_owner_address = blockchain_json.contract_owner.to_string();
    let blockchain_url = blockchain_json.blockchain_url.to_string();
    let blockchain_contract_address;
    if environment == "development" {
        blockchain_contract_address = deploy_contract_locally(
            &blockchain_url,
            blockchain_contract_owner_address.to_owned(),
        )
        .await?;
    } else {
        blockchain_contract_address = blockchain_json.contract_address.to_string();
    }
    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);
    let mut user: User = serde_json::from_str(aux_user).unwrap();
    let user_id = user_service.add(&mut user, &None).await?;
    user_service
        .promote_user_to(&user_id, &PromoteUser::Upgrade)
        .await?;

    println!("update your .env file with this information");
    println!("blockchain url: {}", blockchain_url);
    println!("contract owner: {}", blockchain_contract_owner_address);
    println!("contract address: {}", blockchain_contract_address);
    println!("key id: {}", key_id);
    println!(
        "user admin id: {} device: {}",
        user.user_id(),
        user.device().clone().unwrap()
    );
    println!("secrets:");
    println!("{:?}", secrets_json);
    Ok(())
}

#[tokio::test]
async fn test_all() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use ethers::utils::Ganache;
    use lib_config::infra::build_local_stack_connection;
    use std::env;
    use testcontainers::{clients, images};
    const MNEMONIC_TEST: &str =
        "myth like bonus scare over problem client lizard pioneer submit female collect"; //from $ganache --deterministic command

    env::set_var("RUST_LOG", "debug");
    env::set_var("RUST_BACKTRACE", "full");
    env::set_var("ENVIRONMENT", "development");
    env::set_var(
        "CONTRACT_OWNER_ADDRESS",
        "0x90F8bf6A479f320ead074411a4B0e7944Ea8c9C1",
    );

    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();

    let mut local_stack = images::local_stack::LocalStack::default();
    local_stack.set_services("dynamodb,secretsmanager,kms");
    let node = docker.run(local_stack);
    let host_port = node.get_host_port_ipv4(4566);
    let shared_config = build_local_stack_connection(host_port).await;

    let mut config = Config::new();
    config.setup().await; //read .env file an setup environment.
    config.set_aws_config(&shared_config);

    let ganache = Ganache::new().mnemonic(MNEMONIC_TEST).spawn();
    let blockchain_url = ganache.endpoint();

    let mut new_configuration = config.env_vars().clone();
    new_configuration.set_blockchain_url(blockchain_url.clone());
    config.set_env_vars(&new_configuration);

    non_terraformed_dependencies("development".to_string(), &config).await?;

    config.load_secrets().await;

    Ok(())
}
