use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::config::Config;
use lib_licenses::errors::license::LicenseDynamoDBError;
use lib_licenses::models::license::License;
use lib_licenses::services::licenses::{LicenseManipulation, LicenseService};
use lib_licenses::{
    errors::{
        asset::{AssetDynamoDBError, AssetNoExistsError},
        owner::OwnerNoExistsError,
    },
    services::assets::{AssetManipulation, AssetService},
};
use serde_json::json;
use log::error;
use uuid::Uuid;
use validator::ValidationError;

use crate::my_lambda::{build_resp, build_resp_env, build_resp_no_cache};

//#[instrument]
#[allow(dead_code)]
pub async fn get_my_license(
    _req: &Request,
    _c: &Context,
    config: &Config,
    lic_service: &LicenseService,
    asset_service: &AssetService,
    asset_id: &Uuid,
    user_id: &String,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let op_res = asset_service.get_by_user_asset_id(asset_id, user_id).await;
    match op_res {
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
        Ok(asset) => {
            let lic_op = lic_service.get_by_asset(asset.id()).await;
            match lic_op {
                Err(e) => {
                    if let Some(e) = e.downcast_ref::<LicenseDynamoDBError>() {
                        return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE);
                    } else {
                        return build_resp_env(
                            &config.env_vars().environment().unwrap(),
                            e,
                            StatusCode::INTERNAL_SERVER_ERROR,
                        );
                    }
                }
                Ok(liv) => build_resp_no_cache(json!(liv).to_string(), StatusCode::OK),
            }
        }
    }
}

//#[instrument]
#[allow(dead_code)]
pub async fn get_my_licenses_all(
    _req: &Request,
    _c: &Context,
    config: &Config,
    lic_service: &LicenseService,
    asset_service: &AssetService,
    user_id: &String,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let op_res = asset_service.get_by_user_id(&user_id).await;
    match op_res {
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
        Ok(assets) => {
            let mut res: Vec<License> = vec![];
            for ass in assets.iter() {
                let lic_op = lic_service.get_by_asset(ass.id()).await;
                match lic_op {
                    Err(e) => {
                        error!("{}", e)
                    }
                    Ok(mut lics) => {
                        res.append(&mut lics);
                    }
                }
            }
            build_resp(json!(res).to_string(), StatusCode::OK)
        }
    }
}
