
use lambda_http::RequestExt;
use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::Config;
use lib_licenses::errors::asset::{AssetDynamoDBError, AssetNoExistsError};
use lib_licenses::models::asset::Asset;
use lib_licenses::services::assets::{AssetManipulation, AssetService};
use lib_licenses::services::owners::OwnerService;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use url::Url;
use uuid::Uuid;
use validator::{Validate, ValidationError};
use lib_licenses::services::nfts::NFTsService;

use crate::my_lambda::build_resp;

#[instrument]
pub async fn create_my_nft(
    req: &Request,
    _c: &Context,
    config: &Config,
    asset_service: &AssetService,
    owner_service: &OwnerService,
    blockchain_service: &NFTsService,
    asset_id: &Uuid,
    user_id: &String,
) -> Result<Response<String>, Box<dyn std::error::Error>> {

    let op_res = blockchain_service.add(asset_id, user_id).await;
    match op_res {
        Err(e) => {
            if let Some(m) = e.downcast_ref::<AssetDynamoDBError>() {
                return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<AssetNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else if let Some(m) = e.downcast_ref::<ValidationError>() {
                return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
            } else {
                return build_resp("".to_string(), StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
        Ok(val) => build_resp(val.to_string(), StatusCode::OK),
    }
}
