use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::Config;
use lib_licenses::errors::assets::{DynamoDBError, AssetNoExistsError};
use lib_licenses::services::assets::{AssetManipulation, AssetService};
use serde_json::json;
use tracing::instrument;

use super::build_resp;
use validator::ValidationError;

#[instrument]
pub async fn get_my_asset(
    req: &Request,
    _c: &Context,
    config: &Config,
    asset_service: &AssetService,
    id: &String,
) -> Result<Response<String>, Box<dyn std::error::Error>> {
    let op_res = asset_service.get_by_user_id(&id).await;
    match op_res {
        Ok(user) => build_resp(json!(user).to_string(), StatusCode::OK),
        Err(e) => {
            if let Some(e) = e.downcast_ref::<DynamoDBError>() {
                return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<UserNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else if let Some(m) = e.downcast_ref::<ValidationError>() {
                return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
            } else {
                return build_resp(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }
}

#[instrument]
pub async fn get_my_assets_all(
    req: &Request,
    _c: &Context,
    config: &Config,
    asset_service: &AssetService,
    id: &String,
) -> Result<Response<String>, Box<dyn std::error::Error>> {
    let op_res = asset_service.get_by_user_id(&id).await;
    match op_res {
        Ok(user) => build_resp(json!(user).to_string(), StatusCode::OK),
        Err(e) => {
            if let Some(e) = e.downcast_ref::<DynamoDBError>() {
                return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<UserNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else if let Some(m) = e.downcast_ref::<ValidationError>() {
                return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
            } else {
                return build_resp(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }
}
