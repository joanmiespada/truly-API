
// https://blog.logrocket.com/deploy-lambda-functions-rust/

use lambda_runtime::{run, service_fn, Error };

use lib_config::config::Config;
use lib_licenses::{repositories::{assets::AssetRepo, shorter::ShorterRepo }, services::{ assets::AssetService}};
use my_lambda::{ function_handler};

mod my_lambda;


#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let mut config = Config::new();
    config.setup_with_secrets().await;

    let asset_repo = AssetRepo::new(&config);
    let shorter_repo = ShorterRepo::new(&config);
    let asset_service = AssetService::new(asset_repo,shorter_repo);
    
    run(service_fn(|e| {  function_handler(e,&config, &asset_service) })).await
}