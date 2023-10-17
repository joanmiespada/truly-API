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

//#[instrument]
// pub async fn function_handler(
//     //event: LambdaEvent<SqsEventObj<VideoResult>>,
//     event: LambdaEvent<SqsEventObj<Value>>,
//     _config: &Config,
//     _asset_service: &AssetService,
// ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//     let aux = event.payload.records;
//     for event in aux {
//         let data = event.body;

//         let data_of = data["Message"].to_owned();

//         let content_data = format!("{}", data_of);

//         let mut chars = content_data.chars();
//         chars.next();
//         chars.next_back();
//         let mut res = String::from_str(chars.as_str()).unwrap();

//         res = res.replace("\\n", "");
//         res = res.replace("\\\"", "\"");
//         let op: Result<Value, Error> = serde_json::from_str(&res);
//         match op {
//             Err(e) => {
//                 log::error!("error parsing sqs message!!!!");
//                 log::error!("{}", e);
//                 return Err(e.into());
//             }
//             Ok(data) => {
//                 log::info!("message sqs parsed successfully");
//                 log::info!("{:?}", data);

//             }
//         }
//     }
//     Ok(())
// }

pub async fn function_handler(
    event: LambdaEvent<SqsEventObj<Value>>,
    _config: &Config,
    //_asset_service: &AssetService,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for sqs_record in event.payload.records {
        match serde_json::from_str::<Value>(&sqs_record.body["Message"].to_string()) {
            Err(e) => {
                log::error!("Error parsing SQS message: {}", e);
                // If you don't want to stop processing other records, simply continue here.
                continue;
            }
            Ok(data) => {
                log::info!("Message SQS parsed successfully. Error reported:");
                log::info!("{:?}", data);
            }
        }
    }
    Ok(())
}
