use crate::my_lambda::{build_resp, build_resp_env};
use lambda_http::RequestPayloadExt;
use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::config::Config;
use lib_licenses::errors::asset::{AssetDynamoDBError, AssetNoExistsError};
use lib_licenses::services::assets::{AssetManipulation, AssetService, CreatableFildsAsset};
use lib_licenses::services::video::{VideoService, VideoManipulation};
use tracing::{info, instrument};
use validator::ValidationError;

#[instrument]
pub async fn create_asset(
    req: &Request,
    _c: &Context,
    config: &Config,
    asset_service: &AssetService,
    video_service: &VideoService,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let asset_fields;
    match req.payload::<CreatableFildsAsset>() {
        Err(e) => {
            return build_resp(e.to_string(), StatusCode::BAD_REQUEST);
        }
        Ok(op_payload) => match op_payload {
            None => {
                return build_resp("no payload found".to_string(), StatusCode::BAD_REQUEST);
            }
            Some(payload) => asset_fields = payload.clone(),
        },
    }
    info!("calling asset service: add");
    let op1 = asset_service.add(&asset_fields, &None).await;

    if let Err(e) = op1 {
        if let Some(m) = e.downcast_ref::<AssetDynamoDBError>() {
            return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
        } else if let Some(m) = e.downcast_ref::<AssetNoExistsError>() {
            return build_resp(m.to_string(), StatusCode::NO_CONTENT);
        } else if let Some(m) = e.downcast_ref::<ValidationError>() {
            return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
        } else {
            return build_resp_env(
                &config.env_vars().environment().unwrap(),
                e,
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    }
    let asset_id = op1.ok().unwrap();

    video_service
        .compute_hash_and_similarities_async(&asset_id)
        .await?;

    build_resp(asset_id.to_string(), StatusCode::OK)
}
