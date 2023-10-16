use lib_config::config::Config;
use lib_licenses::services::assets::{AssetManipulation, AssetService};
use lib_hash_objs::hash::HashResult;

//#[instrument]
pub async fn store_after_hash_process(
    data: &HashResult,
    config: &Config,
    asset_service: &AssetService,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let op_res = asset_service.store_hash_process(&data).await;

    match op_res {
        Err(e) => {
            log::error!("{}", e.to_string());
        }
        Ok(_) => {
            log::info!("sucessfully stored");
            log::info!("{:?}", data);
        }
    };
    Ok(())
}
