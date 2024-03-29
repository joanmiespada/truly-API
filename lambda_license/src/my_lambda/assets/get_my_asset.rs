use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::config::Config;
use lib_licenses::services::owners::OwnerService;
use lib_licenses::{
    errors::{
        asset::{AssetDynamoDBError, AssetNoExistsError},
        owner::OwnerNoExistsError,
    },
    services::assets::{AssetManipulation, AssetService},
};
use serde_json::json;
use uuid::Uuid;
use validator::ValidationError;

use lib_util_jwt::build::{build_resp, build_resp_env, build_resp_no_cache};

//#[instrument]
#[allow(dead_code)]
pub async fn get_my_asset(
    _req: &Request,
    _c: &Context,
    config: &Config,
    asset_service: &AssetService,
    _owner_service: &OwnerService,
    asset_id: &Uuid,
    user_id: &String,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let op_res = asset_service.get_by_user_asset_id(asset_id, user_id).await;
    match op_res {
        Ok(assets) => build_resp_no_cache(json!(assets).to_string(), StatusCode::OK),
        Err(e) => {
            if let Some(e) = e.downcast_ref::<AssetDynamoDBError>() {
                return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<AssetNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else if let Some(m) = e.downcast_ref::<OwnerNoExistsError>() {
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
    }
}

//#[instrument]
#[allow(dead_code)]
pub async fn get_my_assets_all(
    _req: &Request,
    _c: &Context,
    config: &Config,
    asset_service: &AssetService,
    _owner_service: &OwnerService,
    user_id: &String,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let op_res = asset_service.get_by_user_id(&user_id).await;
    match op_res {
        Ok(user) => build_resp(json!(user).to_string(), StatusCode::OK),
        Err(e) => {
            if let Some(e) = e.downcast_ref::<AssetDynamoDBError>() {
                return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<AssetNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else if let Some(m) = e.downcast_ref::<OwnerNoExistsError>() {
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
    }
}
