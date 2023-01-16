use lambda_http::service_fn;
use lib_config::Config;
use lib_licenses::repositories::assets::AssetRepo;
use lib_licenses::services::assets::AssetService;
use my_lambda::{ error::ApiLambdaAssetError, function_handler};

mod my_lambda;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let mut config = Config::new();
    config.setup_with_secrets().await;

    let repo = AssetRepo::new(&config);
    let service = AssetService::new(repo);


    let resp = lambda_http::run(service_fn(|event | {
        function_handler(&config, &service, event)
    }))
    .await;

    match resp {
        Ok(r) => Ok(r),
        Err(e) => Err(ApiLambdaAssetError { 0: e.to_string() }.into()),
    }
}
