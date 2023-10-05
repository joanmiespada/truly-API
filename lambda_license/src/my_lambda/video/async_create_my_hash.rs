use lambda_http::RequestPayloadExt;
use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::config::Config;
use lib_licenses::services::assets::AssetService;
use lib_licenses::services::video::{VideoManipulation, VideoService};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::my_lambda::build_resp;

#[derive(Debug, Serialize, Validate, Deserialize, Clone, Copy)]
pub struct CreateHashSimilarsAsync {
    pub asset_id: Uuid,
}

//#[tracing::instrument]
pub async fn async_create_my_hash_similars_sns(
    req: &Request,
    _c: &Context,
    _config: &Config,
    _asset_service: &AssetService,
    video_service: &VideoService,
    _user_id: &String,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let new_hash_similar_video;
    match req.payload::<CreateHashSimilarsAsync>() {
        Err(e) => {
            return build_resp(e.to_string(), StatusCode::BAD_REQUEST);
        }
        Ok(op_payload) => match op_payload {
            None => {
                return build_resp("no payload found".to_string(), StatusCode::BAD_REQUEST);
            }
            Some(payload) => match payload.validate() {
                Err(e) => {
                    return build_resp(e.to_string(), StatusCode::BAD_REQUEST);
                }
                Ok(_) => {
                    new_hash_similar_video = payload.clone();
                }
            },
        },
    }

    let post_request_op = video_service
        .compute_hash_and_similarities_async(&new_hash_similar_video.asset_id)
        .await;

    match post_request_op {
        Err(e) => {
            return build_resp(e.to_string(), StatusCode::CONFLICT);
        }
        Ok(val) => {
            return build_resp(val, StatusCode::OK);
        }
    }
    
}
