use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::config::Config;
use lib_licenses::{services::assets::{AssetManipulation, AssetService}, errors::asset::{AssetDynamoDBError, AssetNoExistsError}};
use lib_licenses::services::owners::OwnerService;
use serde_json::json;
use tracing::instrument;
use uuid::Uuid;
use validator::ValidationError;

use crate::my_lambda::build_resp;

#[instrument]
pub async fn get_asset(
    req: &Request,
    _c: &Context,
    config: &Config,
    asset_service: &AssetService,
    owner_service: &OwnerService,
    asset_id: &Uuid,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let op_res = asset_service.get_by_id(asset_id).await;
    match op_res {
        Ok(assets) => build_resp(json!(assets).to_string(), StatusCode::OK),
        Err(e) => {
            if let Some(e) = e.downcast_ref::<AssetDynamoDBError>() {
                return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<AssetNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else if let Some(m) = e.downcast_ref::<ValidationError>() {
                return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
            } else {
                return build_resp("".to_string(), StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

}



