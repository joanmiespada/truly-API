use lambda_http::service_fn;
use lib_config::config::Config;
use lib_licenses::repositories::block_tx::BlockchainTxRepo;
use lib_licenses::repositories::keypairs::KeyPairRepo;
use lib_licenses::repositories::owners::OwnerRepo;
use lib_licenses::repositories::{assets::AssetRepo, ganache::GanacheRepo};
use lib_licenses::services::assets::AssetService;
use lib_licenses::services::block_tx::BlockchainTxService;
use lib_licenses::services::nfts::NFTsService;
use lib_licenses::services::owners::OwnerService;
use lib_licenses::services::video::VideoService;
use lib_users::repositories::users::UsersRepo;
use lib_users::services::users::UsersService;
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

    let repo_tx = BlockchainTxRepo::new(&config.clone());
    let tx_service = BlockchainTxService::new(repo_tx);

    let asset_repo = AssetRepo::new(&config);
    let asset_service = AssetService::new(asset_repo);

    let owners_repo = OwnerRepo::new(&config);
    let owners_service = OwnerService::new(owners_repo);

    let key_repo = KeyPairRepo::new(&config);
    let blockchain = GanacheRepo::new(&config).unwrap();
    let blockchain_service = NFTsService::new(
        blockchain,
        key_repo,
        asset_service.to_owned(),
        owners_service.to_owned(),
        tx_service.to_owned(),
        config.to_owned(),
    );

    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);

    let video_service = VideoService::new(asset_service.to_owned(), config.to_owned());

    let resp = lambda_http::run(service_fn(|event| {
        function_handler(
            &config,
            &asset_service,
            &owners_service,
            &blockchain_service,
            &user_service,
            &video_service,
            event,
        )
    }))
    .await;

    match resp {
        Ok(r) => Ok(r),
        Err(e) => Err(ApiLambdaError { 0: e.to_string() }.into()),
    }
}
