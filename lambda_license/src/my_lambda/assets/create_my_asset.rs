use lambda_http::RequestExt;
use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::Config;
use lib_licenses::errors::asset::{AssetDynamoDBError, AssetNoExistsError};
use lib_licenses::models::asset::Asset;
use lib_licenses::services::owners::OwnerService;
use lib_licenses::services::assets::{AssetManipulation, AssetService, CreatableFildsAsset};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use url::Url;
use validator::{Validate, ValidationError};

use crate::my_lambda::build_resp;

// #[derive(Debug, Serialize, Validate, Deserialize)]
// pub struct CreateAsset {
//     #[validate(length(max = 100))]
//     pub url: String,
//     #[validate(length(max = 100))]
//     pub license: String,
//     //pub status: Option<String>, //forbidden, only by admins. Roles idem, only admin can change it
//}

#[instrument]
pub async fn create_my_asset(
    req: &Request,
    _c: &Context,
    config: &Config,
    asset_service: &AssetService,
    owner_service: &OwnerService,
    id: &String,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync >> {
    let asset_fields;
    match req.payload::<CreatableFildsAsset>() { //CreateAsset
        Err(e) => {
            return build_resp(e.to_string(), StatusCode::BAD_REQUEST);
        }
        Ok(op_payload) => match op_payload {
            None => {
                return build_resp("no payload found".to_string(), StatusCode::BAD_REQUEST);
            }
            Some(payload) =>  asset_fields = payload.clone()
            
            // match payload.validate() {
            //     Err(e) => {
            //         return build_resp(e.to_string(), StatusCode::BAD_REQUEST);
            //     }
            //     Ok(_) => {
            //         asset_fields = Asset::new();
            //         asset_fields.set_url(&Some(Url::parse(&payload.url).unwrap()));
            //         asset_fields.set_license(&Some(payload.license));
            //     }
            // },
        },
    }

    let op_res = asset_service.add(&asset_fields, id).await;
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
