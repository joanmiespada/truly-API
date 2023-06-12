use lambda_http::service_fn;
use lib_blockchain::blockchains::chain::CloneBoxNFTsRepository;
use lib_blockchain::blockchains::ganache::GanacheBlockChain;
use lib_blockchain::repositories::block_tx::BlockchainTxRepo;
use lib_blockchain::repositories::blockchain::BlockchainRepo;
use lib_blockchain::repositories::contract::ContractRepo;
use lib_blockchain::repositories::keypairs::KeyPairRepo;
use lib_blockchain::services::block_tx::BlockchainTxService;
use lib_blockchain::services::nfts::NFTsService;
use lib_config::config::Config;
use lib_licenses::repositories::licenses::LicenseRepo;
use lib_licenses::repositories::owners::OwnerRepo;
use lib_licenses::repositories::shorter::ShorterRepo;
use lib_licenses::services::assets::AssetService;
use lib_licenses::services::owners::OwnerService;
use lib_licenses::services::video::VideoService;
use lib_licenses::{repositories::assets::AssetRepo, services::licenses::LicenseService};
use lib_users::repositories::users::UsersRepo;
use lib_users::services::users::UsersService;
use my_lambda::{error::ApiLambdaError, function_handler};
use tracing::info;

mod my_lambda;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        //.with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        //.without_time()
        .init();

    info!("bootstrapping dependencies...");

    let mut config = Config::new();
    config.setup_with_secrets().await;

    let repo_tx = BlockchainTxRepo::new(&config.clone());
    let tx_service = BlockchainTxService::new(repo_tx);

    let asset_repo = AssetRepo::new(&config);
    let shorter_repo = ShorterRepo::new(&config);
    let asset_service = AssetService::new(asset_repo.clone(), shorter_repo);

    let owners_repo = OwnerRepo::new(&config);
    let owners_service = OwnerService::new(owners_repo);

    let key_repo = KeyPairRepo::new(&config);

    let blockchains_repo = BlockchainRepo::new(&config);
    let contracts_repo = ContractRepo::new(&config);

    let blockchain = GanacheBlockChain::new(&config, &contracts_repo, &blockchains_repo).await?;

    let blockchain_service = NFTsService::new(
        blockchain.clone_box(),
        key_repo,
        asset_service.to_owned(),
        owners_service.to_owned(),
        tx_service.to_owned(),
        config.to_owned(),
    );

    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);

    let video_service = VideoService::new(asset_service.to_owned(), config.to_owned());

    let license_repo = LicenseRepo::new(&config);
    let license_service = LicenseService::new(license_repo, asset_repo);

    info!("bootstrapping dependencies: completed. Lambda ready.");
    let resp = lambda_http::run(service_fn(|event| {
        function_handler(
            &config,
            &asset_service,
            &owners_service,
            &blockchain_service,
            &user_service,
            &video_service,
            &tx_service,
            &license_service,
            event,
        )
    }))
    .await;

    match resp {
        Ok(r) => Ok(r),
        Err(e) => Err(ApiLambdaError { 0: e.to_string() }.into()),
    }
}
