use admin_user::create_admin_user;
use aws_sdk_dynamodb::types::error::ResourceNotFoundException;
use blockchains::manage_blockchains;
use contracts::manage_contracts;
use lib_config::config::Config;

use schemas::create_schemas;
use secretes::create_secrets;
use serde::{Deserialize, Serialize};
use std::{env, process};
use store_key::create_store_key;
use structopt::StructOpt;
use users::manage_user;

mod admin_user;
mod async_jobs;
mod blockchains;
mod contracts;
mod schemas;
mod secretes;
mod store_key;
mod users;

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
        path,
        //all,
        //async_jobs,
        region,
    }: Opt,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    if let Some(reg) =region {
        env::set_var("AWS_REGION", reg);
    }
    let mut config = Config::new();
    config.setup().await;

    let er = ResourceNotFoundException::builder().build();

    if let Some(table_name) = table {
        //env::set_var("AWS_REGION", region.unwrap());
        //let mut config_multi_region = Config::new();
        //config_multi_region.setup().await;
        create_schemas(table_name.clone(), create, delete, &config).await?;
    }

    if let Some(path) = store_secret {
        create_secrets(create, delete, path, &config).await?;
    }

    if let Some(key_id) = store_key {
        if let Some(key_file_path) = path {
            create_store_key(
                key_id,
                create,
                delete,
                //environment.clone(),
                key_file_path,
                &config,
            )
            .await?;
        } else {
            panic!("key store needs the path of the file!")
        }
    }

    // use aws command line
    // if let Some(_) = key {
    //     if create {
    //         let client_key = aws_sdk_kms::client::Client::new(config.aws_config());
    //         let keyid = create_key(&client_key).await?;

    //         println!("{{'key_id':'{}'}}", keyid)
    //     } else if delete {
    //         panic!("not allowed, do it with AWS console UI")
    //     } else {
    //         return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
    //     }
    // }

    if let Some(email) = adminuser {
        create_admin_user(
            email,
            password,
            create,
            delete,
            environment.clone(),
            &mut config,
        )
        .await?;
    }

    if let Some(id) = user_id {
        manage_user(id, create, delete, environment.clone(), &mut config).await?;
    }

    if let Some(contract_path) = contract {
        manage_contracts(contract_path, create, delete, environment.clone(), &config).await?;
    }

    if let Some(blockchain_path) = blockchain {
        manage_blockchains(
            blockchain_path,
            create,
            delete,
            environment.clone(),
            &config,
        )
        .await?;
    }

    // if let Some(_) = async_jobs {
    //     manage_async_jobs(create, delete, environment.clone(), &config).await?;
    // }

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
    pub store_secret: Option<String>,

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

    #[structopt(long = "path")]
    pub path: Option<String>,
    //#[structopt(long = "all")]
    //pub all: Option<bool>,

    //#[structopt(long = "async")]
    //pub async_jobs: Option<bool>,
    #[structopt(long = "region")]
    pub region: Option<String>,
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
            println!("command executed successfully!")
        }
    }
}
