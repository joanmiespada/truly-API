use admin_user::create_admin_user;
use async_jobs::manage_async_jobs;
use aws_sdk_dynamodb::types::error::ResourceNotFoundException;
use blockchains::manage_blockchains;
use contracts::manage_contracts;
use lib_config::config::Config;
use lib_config::infra::create_key;

use schemas::create_schemas;
use secretes::create_secrets;
use serde::{Deserialize, Serialize};
use users::manage_user;
use std::{env, process};
use store_key::create_store_key;
use structopt::StructOpt;

mod schemas;
mod secretes;
mod store_key;
mod admin_user;
mod users;
mod blockchains;
mod contracts;
mod async_jobs;


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
        blockchain,
        all,
        async_jobs,
    }: Opt,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env::set_var("RUST_LOG", "debug");

    let mut config = Config::new();
    config.setup().await;

    let er = ResourceNotFoundException::builder().build();

    if let Some(table_name) = table {
        create_schemas(table_name, create, delete, &config).await?;
    }

    if let Some(_) = store_secret {
        create_secrets(create, delete, environment.clone(), &config).await?;
    }

    if let Some(key_id) = store_key {
        create_store_key(key_id, create, delete, environment.clone(), &config).await?;
    }

    if let Some(_) = key {
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

    if let Some(email) = adminuser {
        create_admin_user(email,password, create,delete,environment.clone(), &mut config).await?;
    }
    
    if let Some(id) = user_id {
        manage_user(id,create,delete,environment.clone(), &mut config).await?;
    }
    
    if let Some(contract_path) = contract {
        manage_contracts(contract_path, create,delete,environment.clone(), &config).await?;
    }
    

    if let Some(blockchain_path) = blockchain {
       manage_blockchains(blockchain_path, create, delete, environment.clone(), &config).await?;
    }

    if let Some(_) = async_jobs {
        manage_async_jobs(create, delete, environment.clone(), &config).await?;
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

    #[structopt(long = "blockchain")]
    pub blockchain: Option<String>,

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
