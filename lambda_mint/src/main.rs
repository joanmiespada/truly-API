
// https://blog.logrocket.com/deploy-lambda-functions-rust/

use lambda_runtime::{run, service_fn, Error };

use lib_config::config::Config;
use lib_licenses::{repositories::{assets::AssetRepo, owners::OwnerRepo, keypairs::KeyPairRepo, ganache::GanacheRepo, block_tx::BlockchainTxRepo, shorter::ShorterRepo}, services::{owners::OwnerService, assets::AssetService, nfts::NFTsService, block_tx::BlockchainTxService}};
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

    let repo_tx = BlockchainTxRepo::new(&config.clone());
    let tx_service = BlockchainTxService::new(repo_tx);

    let asset_repo = AssetRepo::new(&config);
    let shorter_repo = ShorterRepo::new(&config);
    let asset_service = AssetService::new(asset_repo, shorter_repo);

    let owners_repo = OwnerRepo::new(&config);
    let owners_service = OwnerService::new(owners_repo);

    let key_repo= KeyPairRepo::new(&config);
    let blockchain = GanacheRepo::new(&config).unwrap();
    let blockchain_service = NFTsService::new(
        blockchain,
        key_repo,
        asset_service.to_owned(),
        owners_service.to_owned(),
        tx_service.to_owned(),
        config.to_owned()
    );
    run(service_fn(|e| {  function_handler(e,&config,&blockchain_service) })).await
}