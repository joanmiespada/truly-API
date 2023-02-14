

use tracing::instrument;
use lib_config::config::Config;
use lib_licenses::{services::nfts::{CreateNFTAsync, NFTsService, NFTsManipulation} };


#[instrument]
pub async fn async_minting(
    data: &CreateNFTAsync,
    config: &Config,
    blockchain_service: &NFTsService,
) -> Result<(),Box<dyn std::error::Error + Send + Sync>> {


    let op_res = blockchain_service.try_mint(
        &data.asset_id,
        &data.user_id,
        &data.price).await;
    
    match op_res {
        Err(e) => {
                println!("{}", e.to_string());
        },
        Ok(tx) => {
            println!("{}",tx)
        },
      };
      Ok( () )
}


