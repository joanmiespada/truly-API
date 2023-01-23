use lambda_http::service_fn;
use lib_config::Config;
use lib_licenses::repositories::owners::OwnerRepo;
use lib_licenses::repositories::{assets::AssetRepo, ganache::GanacheRepo};
use lib_licenses::services::assets::AssetService;
use lib_licenses::services::nfts::NFTsService;
use lib_licenses::services::owners::OwnerService;
use my_lambda::{error::ApiLambdaError, function_handler};

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

    let asset_repo = AssetRepo::new(&config);
    let asset_service = AssetService::new(asset_repo);

    let owners_repo = OwnerRepo::new(&config);
    let owners_service = OwnerService::new(owners_repo);

    let blockchain = GanacheRepo::new(&config);
    let blockchain_service = NFTsService::new(blockchain);

    let resp = lambda_http::run(service_fn(|event| {
        function_handler(
            &config, 
            &asset_service, 
            &owners_service,
            &blockchain_service, 
            event)
    }))
    .await;

    match resp {
        Ok(r) => Ok(r),
        Err(e) => Err(ApiLambdaError { 0: e.to_string() }.into()),
    }
}