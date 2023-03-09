

use tracing::instrument;
use lib_config::config::Config;
use lib_licenses::{services::nfts::{CreateNFTAsync, NFTsService, NFTsManipulation} };


#[instrument]
pub async fn store_after_video_process(
    data: &VideoResult,
    config: &Config,
    asset_service: &AssetService,
) -> Result<(),Box<dyn std::error::Error + Send + Sync>> {


    let op_res = asset_service.store_video_process(&data).await;
    
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


