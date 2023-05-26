use crate::my_lambda::{build_resp, build_resp_env};
use lambda_http::RequestPayloadExt;
use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::config::Config;
use lib_licenses::errors::asset::AssetNoExistsError;
use lib_licenses::errors::license::LicenseDynamoDBError;
use lib_licenses::errors::owner::OwnerNoExistsError;
use lib_licenses::models::license::CreatableFildsLicense;
use lib_licenses::services::licenses::{LicenseManipulation, LicenseService};
use tracing::{info, instrument};
use validator::ValidationError;

#[instrument]
pub async fn create_my_license(
    req: &Request,
    _c: &Context,
    config: &Config,
    lic_service: &LicenseService,
    user_id: &String,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let lic_fields;
    match req.payload::<CreatableFildsLicense>() {
        Err(e) => {
            return build_resp(e.to_string(), StatusCode::BAD_REQUEST);
        }
        Ok(op_payload) => match op_payload {
            None => {
                return build_resp("no payload found".to_string(), StatusCode::BAD_REQUEST);
            }
            Some(payload) => lic_fields = payload.clone(),
        },
    }

    info!("calling license service: add");
    let op_res = lic_service.create(&lic_fields, &Some(user_id.to_string())).await;
    match op_res {
        Err(e) => {
            if let Some(m) = e.downcast_ref::<LicenseDynamoDBError>() {
                return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<AssetNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else if let Some(m) = e.downcast_ref::<ValidationError>() {
                return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
            } else if let Some(m) = e.downcast_ref::<OwnerNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
            } else {
                return build_resp_env(
                    config.env_vars().environment(),
                    e,
                    StatusCode::INTERNAL_SERVER_ERROR,
                );
            }
        }
        Ok(val) => build_resp(val.to_string(), StatusCode::OK),
    }
}
