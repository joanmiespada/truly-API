mod after_hash;

use std::str::FromStr;

use crate::my_lambda::after_hash::store_after_hash_process_successfully;
use aws_lambda_events::sqs::SqsEventObj;
use lambda_runtime::LambdaEvent;
use lib_config::config::Config;
use lib_licenses::services::assets::AssetService;
use lib_hash_objs::hash::HashResult;
use serde_json::{Error, Value};

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
//#[instrument]
pub async fn function_handler(
    //event: LambdaEvent<SqsEventObj<VideoResult>>,
    event: LambdaEvent<SqsEventObj<Value>>,
    config: &Config,
    asset_service: &AssetService,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let aux = event.payload.records;
    for event in aux {
        let data = event.body;

        let data_of = data["Message"].to_owned();

        let content_data = format!("{}", data_of);

        let mut chars = content_data.chars();
        chars.next();
        chars.next_back();
        let mut res = String::from_str(chars.as_str()).unwrap();

        res = res.replace("\\n", "");
        res = res.replace("\\\"", "\"");
        let op: Result<HashResult, Error> = serde_json::from_str(&res);
        match op {
            Err(e) => {
                log::error!("error parsing sqs message!!!!");
                log::error!("{}", e);
                return Err(e.into());
            }
            Ok(data) => {
                log::info!("message sqs parsed successfully");
                println!("{:?}", data);

                store_after_hash_process_successfully(&data, config, asset_service).await?;
            }
        }
    }
    Ok(())
}
