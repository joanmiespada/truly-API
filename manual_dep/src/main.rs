use aws_sdk_dynamodb::error::ResourceNotFoundException;
use lib_config::config::Config;
use lib_config::infra::{
    create_key, create_secret_manager_keys, create_secret_manager_secret_key, store_secret_key,
};
use lib_licenses::repositories::{schema_asset, schema_keypairs, schema_owners};
use lib_licenses::services::contract::deploy_contract_locally;
use lib_users::models::user::User;
use lib_users::repositories::schema_user;
use lib_users::repositories::users::UsersRepo;
use lib_users::services::users::PromoteUser;
use lib_users::services::users::{UserManipulation, UsersService};
use serde_json::Value;
use std::{env, process};
use structopt::StructOpt;

#[allow(unused_variables)]
async fn command(
    Opt {
        table,
        create,
        delete,
        environment,
        store_secret,
        store_key,
        key,
        user,
        contract,
        all,
    }: Opt,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env::set_var("RUST_LOG", "debug");

    let mut config = Config::new();
    config.setup().await;

    let er = ResourceNotFoundException::builder().build();

    match table {
        None => {}
        Some(table_name) => {
            let client = aws_sdk_dynamodb::Client::new(config.aws_config());
            match table_name.as_str() {
                "owners" => {
                    if create {
                        schema_owners::create_schema_owners(&client).await?
                    } else if delete {
                        schema_owners::delete_schema_owners(&client).await?
                    } else {
                        return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
                    }
                }
                "assets" => {
                    if create {
                        schema_asset::create_schema_assets(&client).await?
                    } else if delete {
                        schema_asset::delete_schema_assets(&client).await?
                    } else {
                        return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
                    }
                }
                "keypairs" => {
                    if create {
                        schema_keypairs::create_schema_keypairs(&client).await?
                    } else if delete {
                        schema_keypairs::delete_schema_keypairs(&client).await?
                    } else {
                        return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
                    }
                }
                "users" => {
                    if create {
                        schema_user::create_schema_users(&client).await?
                    } else if delete {
                        schema_user::delete_schema_users(&client).await?
                    } else {
                        return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
                    }
                }
                _ => {
                    return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
                }
            }
        }
    }
    match store_secret {
        None => {}
        Some(scretes_ok) => {
            if create {
                let client_sec = aws_sdk_secretsmanager::client::Client::new(config.aws_config());
                let secrets_json;
                if environment == "development" {
                    secrets_json = include_str!("../res/secrets_development.json");
                } else {
                    secrets_json = include_str!("../res/secrets_prod_stage.json");
                }
                create_secret_manager_keys(secrets_json, &client_sec).await?;
            } else if delete {
                panic!("not allowed, do it with AWS console UI")
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
    }
    match store_key {
        None => {}
        Some(key_id) => {
            if create {
                let client_sec = aws_sdk_secretsmanager::client::Client::new(config.aws_config());
                let secret_key_raw;
                if environment == "development" {
                    secret_key_raw = include_str!("../res/key_development.txt");
                } else {
                    secret_key_raw = include_str!("../res/key_prod_stage.txt");
                }

                let res_op = create_secret_manager_secret_key(&client_sec).await;
                match res_op {
                    Err(e) => {
                        panic!("{}", e.to_string())
                    }
                    Ok(_) => match store_secret_key(&secret_key_raw, &key_id, &config).await {
                        Err(e) => panic!("{}", e.to_string()),
                        Ok(_) => {}
                    },
                }
            } else if delete {
                panic!("not allowed, do it with AWS console UI")
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
    }
    match key {
        None => {}
        Some(keys_ok) => {
            if create {
                let client_key = aws_sdk_kms::client::Client::new(config.aws_config());
                let keyid = create_key(&client_key).await?;
                println!("new keyid : {}", keyid)
            } else if delete {
                panic!("not allowed, do it with AWS console UI")
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
    }
    match user {
        None => {}
        Some(_) => {
            let aux_user;
            if environment == "development" {
                aux_user = include_str!("../res/user_development.json");
            } else {
                aux_user = include_str!("../res/user_prod_stage.json");
            }
            let user_repo = UsersRepo::new(&config);
            let user_service = UsersService::new(user_repo);

            let mut user: User = serde_json::from_str(aux_user).unwrap();

            user_service.add_user(&mut user, &None).await?;
        }
    }
    match contract {
        None => {}
        Some(url) => {
            let contract_owner_address = "0x90F8bf6A479f320ead074411a4B0e7944Ea8c9C1".to_string(); //account[0] from ganache --deterministic
            let address = deploy_contract_locally(&url, contract_owner_address).await?;
            println!("contract address deployed at: {}", address);
        }
    }
    match all {
        None => {}
        Some(_) => {
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
                aux_user = include_str!("../res/user_development.json");
            } else {
                aux_user = include_str!("../res/user_prod_stage.json");
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

            let blockchain_json: Value = serde_json::from_str(blockchain).unwrap();
            let blockchain_contract_owner_address = blockchain_json["contract_owner"].to_string();
            let blockchain_url = blockchain_json["blockchain_url"].to_string();
            let blockchain_contract_address;
            if environment == "development" {
                blockchain_contract_address = deploy_contract_locally(
                    &blockchain_url,
                    blockchain_contract_owner_address.to_owned(),
                )
                .await?;
            } else {
                blockchain_contract_address = blockchain_json["contract_address"].to_string();
            }
            let user_repo = UsersRepo::new(&config);
            let user_service = UsersService::new(user_repo);
            let mut user: User = serde_json::from_str(aux_user).unwrap();
            let user_id = user_service.add_user(&mut user, &None).await?;
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
        }
    }

    Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "non-terraform dependencies creation/delete",
    about = "Create and Destroy dependencies"
)]
pub struct Opt {
    #[structopt(long = "table")]
    pub table: Option<String>,

    #[structopt(long = "create")]
    pub create: bool,

    #[structopt(long = "delete")]
    pub delete: bool,

    #[structopt(env = "ENVIRONMENT")]
    pub environment: String,

    #[structopt(long = "store_secret")]
    pub store_secret: Option<bool>,

    #[structopt(long = "store_key")]
    pub store_key: Option<String>,

    #[structopt(long = "key")]
    pub key: Option<bool>,

    #[structopt(long = "user")]
    pub user: Option<bool>,

    #[structopt(long = "contract")]
    pub contract: Option<String>,

    #[structopt(long = "all")]
    pub all: Option<bool>,
}

#[tokio::main]
async fn main() {
    let op_cmd = command(Opt::from_args()).await;
    match op_cmd {
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
        Ok(_) => {
            println!("successful!")
        }
    }
}
