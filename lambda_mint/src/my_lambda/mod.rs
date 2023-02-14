mod mint_nft_async;

use aws_lambda_events::sqs::SqsEventObj;
use lambda_http::{
     http::Method, http::StatusCode, lambda_runtime::Context,
      IntoResponse, Request, RequestExt, Response,
};
use lambda_runtime::LambdaEvent;
use serde_json::json;
use tracing::{instrument};
use lib_config::config::Config;
use lib_licenses::services::nfts::{NFTsService, CreateNFTAsync};

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
    event: LambdaEvent<SqsEventObj<CreateNFTAsync>>,
    config: &Config,
    blockchain_service: &NFTsService,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    //let context = req.lambda_context();Box<dyn std::error::Error + Send + Sync
    //let query_string = req.query_string_parameters().to_owned();
    //request.uri().path()

    let data = &event.payload.records[0].body;

            
    async_minting(&data, config, blockchain_service).await
   // match req.method() {
    //    &Method::POST => {
        
    //    } 
        // match req.uri().path() {
        //     "/auth/login" =>  login(&req, &context, config, user_service).await,
        //     "/auth/signup" => create_basic_user(&req, &context, config, user_service).await,
        //     &_ => build_resp(
        //         "method not allowed".to_string(),
        //         StatusCode::METHOD_NOT_ALLOWED,
        //     ),
        // },
    //    _ => Err(ApiLambdaError { 0: "not matching".to_string() }.into()),
   // }
}
