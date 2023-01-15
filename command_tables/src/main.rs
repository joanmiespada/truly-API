use std::process;
use aws_sdk_dynamodb::{Error};
use lib_config::Config;
use lib_licenses::repositories::{ schema_owners, schema_asset };
use aws_sdk_dynamodb::error::ResourceNotFoundException;


async fn command(Opt { table }: Opt) -> Result<(), Error> {
    let mut config = Config::new();
    config.setup().await;
    let er = ResourceNotFoundException::builder().build();
    match table.unwrap().as_str() {
        "owners" => schema_owners::create_schema_owners(&config).await?,
        "assets" => schema_asset::create_schema_assets(&config).await?,
        _ =>{  return Err( aws_sdk_dynamodb::Error::ResourceNotFoundException(er) );} 
    }

    Ok(())
}

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    pub table: Option<String>,

    //#[structopt(short, long)]
    //pub verbose: bool,
}


#[tokio::main]
async fn main() {


    if let Err(err) = command(Opt::from_args()).await {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}

