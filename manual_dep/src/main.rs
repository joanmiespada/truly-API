use aws_sdk_dynamodb::error::ResourceNotFoundException;
use lib_async_ops::sns::create as create_topic;
use lib_async_ops::sqs::create as create_queue;
use lib_config::config::Config;
use lib_config::infra::{
    create_key, create_secret_manager_keys, create_secret_manager_secret_key, store_secret_key,
};
use lib_licenses::repositories::{schema_asset, schema_owners};
use lib_blockchain::repositories::{schema_block_tx, schema_keypairs, schema_blockchain, schema_contract};
use lib_blockchain::services::contract::deploy_contract_locally;
use lib_users::models::user::User;
use lib_users::repositories::schema_user;
use lib_users::repositories::users::UsersRepo;
use lib_users::services::users::{PromoteUser, UserManipulation, UsersService};
use serde::{Deserialize, Serialize};
use std::{env, process};
use structopt::StructOpt;


#[derive(Debug, Serialize, Deserialize)]
struct NewUser {
    pub email: String,
    pub device: String,
}

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
        adminuser,
        user_id,
        password,
        contract,
        all,
        async_jobs,
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
                        schema_asset::create_schema_assets_all(&client).await?;
                    } else if delete {
                        schema_asset::delete_schema_assets_all(&client).await?;
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
                "transactions" => {
                    if create {
                        schema_block_tx::create_schema_transactions(&client).await?
                    } else if delete {
                        schema_block_tx::delete_schema_transactions(&client).await?
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
                "blockchains" => {
                    if create {
                        schema_blockchain::create_schema_blockchains(&client).await?;
                    } else if delete {
                        schema_blockchain::delete_schema_blockchains(&client).await?;
                    } else {
                        return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
                    }
                }
                "contracts" => {
                    if create {
                        schema_contract::create_schema_contracts(&client).await?;
                    } else if delete {
                        schema_contract::delete_schema_contracts(&client).await?;
                    } else {
                        return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
                    }
                }
                "all" =>{
                    if create {
                        schema_blockchain::create_schema_blockchains(&client).await?;
                        schema_contract::create_schema_contracts(&client).await?;
                        schema_owners::create_schema_owners(&client).await?;
                        schema_asset::create_schema_assets_all(&client).await?;
                        schema_keypairs::create_schema_keypairs(&client).await?;
                        schema_block_tx::create_schema_transactions(&client).await?;
                        schema_user::create_schema_users(&client).await?;
                    } else if delete{
                        schema_blockchain::delete_schema_blockchains(&client).await?;
                        schema_contract::delete_schema_contracts(&client).await?;
                        schema_owners::delete_schema_owners(&client).await?;
                        schema_asset::delete_schema_assets_all(&client).await?;
                        schema_keypairs::delete_schema_keypairs(&client).await?;
                        schema_block_tx::delete_schema_transactions(&client).await?;
                        schema_user::delete_schema_users(&client).await?;
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
    match adminuser {
        None => {}
        Some(email) => {
            config.load_secrets().await;
            let user_repo = UsersRepo::new(&config);
            let user_service = UsersService::new(user_repo);
            if create {
                
                let mut user = User::new();
                user.set_email(&email);
                let device = uuid::Uuid::new_v4().to_string();
                user.set_device(&device);

                let user_id = user_service.add(&mut user, &password).await?;
                user_service
                    .promote_user_to(&user_id, &PromoteUser::Upgrade)
                    .await?;
                println!("admin user id:{} with device: {} created.", user_id, device);
            } else {
                println!("Not implemented yet")
            }
        }
    }
    match user_id{
        None=>{},
        Some(id)=>{
            config.load_secrets().await;
            let user_repo = UsersRepo::new(&config);
            let user_service = UsersService::new(user_repo);
            if delete {
                let op = user_service.remove_by_id(&id).await;
                match op {
                    Err(e) => {
                        println!("{}", e);
                    }
                    Ok(_) => {
                        println!("user {} deleted!", id)
                    }
                }
            } else {
                println!("Not implemented yet")
            }
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
    
    match async_jobs {
        None => {}
        Some(keys_ok) => {
            if create {
                let name1 = "queue_minting_async".to_string();
                let url1 = create_queue(&config, name1.to_owned()).await?;
                println!("queue {} created at url: {}", name1, url1);

                let name2 = "queue_minting_deathletter".to_string();
                let url2 = create_queue(&config, name2.to_owned()).await?;
                println!("queue {} created at url: {}", name2, url2);

                let name3 = "topic_minting_async".to_string();
                let arn = create_topic(&config, name3.to_owned()).await?;
                println!("topic {} created at arn: {}", name2, arn);
            } else if delete {
                panic!("not implemented yet")
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
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

    #[structopt(long = "adminuser")]
    pub adminuser: Option<String>,
    
    #[structopt(long = "user_id")]
    pub user_id: Option<String>,

    #[structopt(long = "password")]
    pub password: Option<String>,

    #[structopt(long = "contract")]
    pub contract: Option<String>,

    #[structopt(long = "all")]
    pub all: Option<bool>,

    #[structopt(long = "async")]
    pub async_jobs: Option<bool>,
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "debug");
    env_logger::builder().is_test(true).init();

    let op_cmd = command(Opt::from_args()).await;
    match op_cmd {
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
        Ok(_) => {
            println!("command executed successfully!")
        }
    }
}
