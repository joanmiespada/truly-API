mod mint_nft_async;

use std::str::FromStr;

use aws_lambda_events::sqs::SqsEventObj;
use lambda_runtime::LambdaEvent;
use lib_config::config::Config;
use lib_licenses::services::nfts::{CreateNFTAsync, NFTsService};
use serde_json::{Error,Value};
use tracing::{instrument, error, info};

use self::mint_nft_async::async_minting;

#[derive(Debug)]
pub struct ApiLambdaError(pub String);

impl std::error::Error for ApiLambdaError {}

impl std::fmt::Display for ApiLambdaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "api lambda error: {}", self.0)
    }
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
#[instrument]
pub async fn function_handler(
    //event: LambdaEvent<SqsEventObj<CreateNFTAsync>>,
    event: LambdaEvent<SqsEventObj<Value>>,
    config: &Config,
    blockchain_service: &NFTsService,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let aux = event.payload.records;
    for event in aux {
        let data = event.body;

        let shorter_of = data["Message"].to_owned();

        let aux = format!("{}", shorter_of);

        let mut chars = aux.chars();
        chars.next();
        chars.next_back();
        let mut res = String::from_str(chars.as_str()).unwrap();

        res = res.replace("\\n", "");
        res = res.replace("\\\"", "\"");
        let op: Result<CreateNFTAsync, Error> = serde_json::from_str(&res);
        match op {
            Err(e) => {
                error!("error parsing sqs message!!!!");
                error!("{}", e);
                return Err(e.into());
            }
            Ok(data) => {
                info!("message sqs parsed successfully");
                println!("{}", data);
                let mut aux = data.clone();
                async_minting(&mut aux, config, blockchain_service).await?;
            }
        }
    }
    Ok(())
}
