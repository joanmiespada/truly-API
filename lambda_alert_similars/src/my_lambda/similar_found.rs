use lib_config::config::Config;
use lib_hash_objs::similar_alert::AlertExternalPayload;
use lib_licenses::{services::alert_similar::AlertSimilarService, repositories::alert_similar::AlertSimilarRepo, models::alert_similar::AlertSimilarBuilder};

//#[instrument]
pub async fn store_similar_found_successfully(
    data: &AlertExternalPayload,
    _config: &Config,
    notification_service: &AlertSimilarService<AlertSimilarRepo>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let mut notif_build = AlertSimilarBuilder::default();
    notif_build
        .source_type(data.source_type.clone())
        .origin_hash_id(data.origin_hash_id.clone())
        .origin_hash_type(data.origin_hash_type.clone())
        .origin_frame_id(data.origin_frame_id.clone())
        .origin_frame_second(data.origin_frame_second.clone())
        .origin_frame_url(data.origin_frame_url.clone())
        .origin_asset_id(data.origin_asset_id.clone())
        .similar_frame_id(data.similar_frame_id.clone())
        .similar_frame_second(data.similar_frame_second.clone())
        .similar_frame_url(data.similar_frame_url.clone())
        .similar_asset_id(data.similar_asset_id.clone());

    let op_res = notification_service.add(&mut notif_build).await;

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
