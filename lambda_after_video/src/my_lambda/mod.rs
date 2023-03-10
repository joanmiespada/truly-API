mod after_video;

use aws_lambda_events::sqs::SqsEventObj;
use lambda_runtime::LambdaEvent;
use tracing::instrument;
use lib_config::config::Config;
use lib_licenses::models::video::VideoResult;
use lib_licenses::services::assets::AssetService;
use crate::my_lambda::after_video::store_after_video_process;


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
    event: LambdaEvent<SqsEventObj<VideoResult>>,
    config: &Config,
    asset_service: &AssetService,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let aux = event.payload.records;
    for event in aux { 
        let data = event.body;
        store_after_video_process(&data, config, asset_service).await?;
    }
    Ok(())
}
