

use tracing::{instrument,error,info};
use lib_config::config::Config;
use lib_licenses::models::video::VideoResult;
use lib_licenses::services::assets::{AssetService, AssetManipulation };


#[instrument]
pub async fn store_after_video_process(
    data: &VideoResult,
    config: &Config,
    asset_service: &AssetService,
) -> Result<(),Box<dyn std::error::Error + Send + Sync>> {


    let op_res = asset_service.store_video_process(&data).await;
    
    match op_res {
        Err(e) => {
                error!("{}", e.to_string());
        },
        Ok(_) => { 
            info!("sucessfully stored");
            info!("{:?}",data);
        },
      };
      Ok( () )
}


