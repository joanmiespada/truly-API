use aws_lambda_events::sqs::SqsEventObj;
use lambda_runtime::LambdaEvent;
use lib_config::config::Config;
use serde_json::Value;


pub async fn function_handler(
    event: LambdaEvent<SqsEventObj<Value>>,
    _config: &Config,
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

        match serde_json::from_str::<Value>(sns_message_str) {
            Err(e) => {
                log::error!("Error parsing SNS message from SQS: {}", e);
                continue;
            },
            Ok(data) => {
                log::info!("Message from SNS parsed successfully");
                log::info!("{:?}", data);
            }
        }
    }
    Ok(()) 
}
