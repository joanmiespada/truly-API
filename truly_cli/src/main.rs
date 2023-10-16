use admin_user::create_admin_user;
use aws_sdk_dynamodb::types::error::ResourceNotFoundException;
//use blockchains::manage_blockchains;
//use contracts::manage_contracts;
//use ledger::manage_ledger;
use lib_config::config::Config;

use schemas::create_schemas;
use secrets::create_secrets;
use serde::{Deserialize, Serialize};
use std::{env, process};
//use store_key::create_store_key;
use structopt::StructOpt;
use users::manage_user;

mod admin_user;
mod async_jobs;
//mod blockchains;
//mod contracts;
mod schemas;
mod secrets;
mod users;
mod ledger;

#[derive(Debug, Serialize, Deserialize)]
struct NewUser {
    pub email: String,
    pub device: String,
}

#[allow(unused_variables)]
async fn command(
    Opt {
        service,
        create,
        delete,
        environment,
        store_secret,
        key,
        adminuser,
        user_id,
        password,
        // contract,
        // blockchain,
        //ledger,
        region,
        profile,
    }: Opt,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(reg) = region {
        env::set_var("AWS_REGION", reg);
    }
    if let Some(prof) = profile {
        env::set_var("AWS_PROFILE", prof);
    }
    let mut config = Config::new();
    config.setup().await;

    let er = ResourceNotFoundException::builder().build();

    if let Some(service_name) = service {
        create_schemas(service_name.clone(), create, delete, &config).await?;
    }

    if let Some(path) = store_secret {
        create_secrets(create, delete, path, &config).await?;
    }

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

    // if let Some(contract_path) = contract {
    //     manage_contracts(contract_path, create, delete, environment.clone(), &config).await?;
    // }

    // if let Some(blockchain_path) = blockchain {
    //     manage_blockchains(
    //         blockchain_path,
    //         create,
    //         delete,
    //         environment.clone(),
    //         &config,
    //     )
    //     .await?;
    // }

    // if let Some(_) = ledger {
    //      manage_ledger(create, delete,  &config).await?;
    //  }

    Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "non-terraform dependencies creation/delete",
    about = "Create and Destroy dependencies"
)]
pub struct Opt {
    #[structopt(long = "service")]
    pub service: Option<String>,

    #[structopt(long = "create")]
    pub create: bool,

    #[structopt(long = "delete")]
    pub delete: bool,

    #[structopt(env = "ENVIRONMENT")]
    pub environment: String,

    #[structopt(long = "store_secret")]
    pub store_secret: Option<String>,

    //#[structopt(long = "store_key")]
    //pub store_key: Option<String>,
    #[structopt(long = "key")]
    pub key: Option<bool>,

    #[structopt(long = "adminuser")]
    pub adminuser: Option<String>,

    #[structopt(long = "user_id")]
    pub user_id: Option<String>,

    #[structopt(long = "password")]
    pub password: Option<String>,

    // #[structopt(long = "contract")]
    // pub contract: Option<String>,

    // #[structopt(long = "blockchain")]
    // pub blockchain: Option<String>,

    // #[structopt(long = "ledger")]
    // pub ledger: Option<bool>,

    #[structopt(long = "region")]
    pub region: Option<String>,

    #[structopt(long = "profile")]
    pub profile: Option<String>,
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
