use aws_sdk_dynamodb::error::ResourceNotFoundException;
use aws_sdk_dynamodb::{Error, Client};
use lib_config::Config;
use lib_licenses::repositories::schema_asset;
use structopt::StructOpt;
use std::process ;

async fn command(
    Opt {
        table,
        create,
        delete,
        environment
    }: Opt,
) -> Result<(), Error> {
    let mut config = Config::new();
    config.setup().await;
    let client = Client::new(config.aws_config());
    
    let er = ResourceNotFoundException::builder().build();
    
    match table.unwrap().as_str() {
        /* "owners" => {
            if create {
                schema_owners::create_schema_owners(&config).await?
            } else if delete {
                schema_owners::delete_schema_owners(&config).await?
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er));
            }
        } */
        "assets" => {
            if create {
                schema_asset::create_schema_assets(&client).await?
            } else if delete {
                schema_asset::delete_schema_assets(&client).await?
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er));
            }
        }
        _ => {
            return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er));
        }
    }

    Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "table creation",
    about = "Create and Destroy tables on Dynamodb"
)]
pub struct Opt {
    #[structopt(short = "t", long = "table")]
    pub table: Option<String>,

    #[structopt(short = "c", long = "create")]
    pub create: bool,

    #[structopt(short = "d", long = "delete")]
    pub delete: bool,

    #[structopt(env="ENVIRONMENT")]
    pub environment: String
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