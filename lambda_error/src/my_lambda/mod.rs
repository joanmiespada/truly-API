use aws_lambda_events::sqs::SqsEventObj;
use lambda_runtime::LambdaEvent;
use lib_config::config::Config;
//use lib_licenses::services::assets::AssetService;
use serde_json::Value;

#[derive(Debug)]
pub struct ApiLambdaError(pub String);

impl std::error::Error for ApiLambdaError {}

impl std::fmt::Display for ApiLambdaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "api lambda error: {}", self.0)
    }
}

pub async fn function_handler(
    event: LambdaEvent<SqsEventObj<Value>>,
    _config: &Config,
    //_asset_service: &AssetService,
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
        match serde_json::from_str::<Value>(sns_message_str) {
            Err(e) => {
                log::error!("Error parsing SNS message from SQS: {}", e);
                continue;
            },
            Ok(data) => {
                log::info!("Message from SNS parsed successfully");
                log::info!("{:?}", data);
                // Process the parsed data as needed...
            }
        }
    }
    Ok(()) 
}
