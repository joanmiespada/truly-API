mod after_hash;
use crate::my_lambda::after_hash::store_after_hash_process_successfully;
use aws_lambda_events::sqs::SqsEventObj;
use lambda_runtime::LambdaEvent;
use lib_config::config::Config;
use lib_licenses::services::assets::AssetService;
use lib_hash_objs::hash::HashResult;
use serde_json::Value;

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

    for sqs_record in &event.payload.records {
        // Extract the SNS message from SQS message body
        let sns_message_str: &str = match sqs_record.body.get("Message") {
            Some(message) => message.as_str().unwrap_or_default(),
            None => {
                log::error!("Missing 'Message' attribute in SQS record.");
                continue;
            }
        };

        // Deserialize the SNS message
        match serde_json::from_str::<HashResult>(sns_message_str) {
            Err(e) => {
                log::error!("Error parsing SNS message from SQS: {}", e);
                continue;
            },
            Ok(data) => {
                log::info!("Message from SNS parsed successfully");
                log::info!("{:?}", data);
                store_after_hash_process_successfully(&data, config, asset_service).await?;
                // Process the parsed data as needed...
            }
        }
    }
    Ok(())





    // let aux = event.payload.records;
    // for event in aux {
    //     let sns_data = event.body;

    //     let data_of = sns_data["Message"].to_owned();

    //     let content_data = format!("{}", data_of);

    //     let mut chars = content_data.chars();
    //     chars.next();
    //     chars.next_back();
    //     let mut res = String::from_str(chars.as_str()).unwrap();

    //     res = res.replace("\\n", "");
    //     res = res.replace("\\\"", "\"");
    //     let op: Result<HashResult, Error> = serde_json::from_str(&res);
    //     match op {
    //         Err(e) => {
    //             log::error!("error parsing sqs message!!!!");
    //             log::error!("{}", e);
    //             return Err(e.into());
    //         }
    //         Ok(data) => {
    //             log::info!("message sqs parsed successfully");
    //             println!("{:?}", data);

    //             store_after_hash_process_successfully(&data, config, asset_service).await?;
    //         }
    //     }
    // }
    // Ok(())
}
