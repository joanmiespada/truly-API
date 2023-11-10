mod after_hash;
use crate::my_lambda::after_hash::store_after_hash_process_successfully;
use aws_lambda_events::sqs::SqsEventObj;
use lambda_runtime::LambdaEvent;
use lib_config::config::Config;
use lib_licenses::services::assets::AssetService;
use lib_hash_objs::hash::HashResult;
use serde_json::Value;


//#[instrument]
pub async fn function_handler(
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
   
}
