use crate::{
    models::asset::HashProcessStatus,
    services::{
        assets::{AssetManipulation, AssetService, CreatableFildsAsset},
        video::{VideoManipulation, VideoService},
    },
};
use lib_config::result::ResultE;
use uuid::Uuid;

pub async fn create_asset(
    asset_service: &AssetService,
    video_service: &VideoService,
    user_id: Option<String>,
    asset_fields: &CreatableFildsAsset,
) -> ResultE<Uuid> {
    let op1 = asset_service.add(&asset_fields, &user_id).await;

    match op1 {
        Ok(asset_id) => {
            video_service
                .compute_hash_and_similarities_async(&asset_id)
                .await?;

            let mut ass = asset_service.get_by_id(&asset_id).await?;
            ass.set_hash_process_status(&Some(HashProcessStatus::Started));
            asset_service.update_full(&ass).await?;
            Ok(asset_id)
        }
        Err(e) => Err(e),
    }
}
