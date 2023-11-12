
pub mod my_lambda;
use crate::my_lambda::similar_found::store_similar_found_successfully;
use aws_lambda_events::sqs::SqsEventObj;
use lambda_runtime::LambdaEvent;
use lib_config::config::Config;
use lib_hash_objs::similar_alert::AlertExternalPayload;
use serde_json::Value;
use lib_engage::{services::alert_similar::AlertSimilarService, repositories::alert_similar::AlertSimilarRepo};


//#[instrument]
pub async fn function_handler(
    //event: LambdaEvent<SqsEventObj<VideoResult>>,
    event: LambdaEvent<SqsEventObj<Value>>,
    config: &Config,
    notification_service: &AlertSimilarService<AlertSimilarRepo>,
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
        match serde_json::from_str::<AlertExternalPayload>(sns_message_str) {
            Err(e) => {
                log::error!("Error parsing SNS message from SQS: {}", e);
                continue;
            },
            Ok(data) => {
                log::info!("Message from SNS parsed successfully");
                log::info!("{:?}", data);
                store_similar_found_successfully(&data, config, notification_service).await?;
                // Process the parsed data as needed...
            }
        }
    }
    Ok(())
    
}
