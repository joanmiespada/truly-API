use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lambda_http::RequestPayloadExt;
use lib_config::config::Config;
use lib_licenses::services::video::{VideoManipulation, VideoService};
use lib_licenses::{
    errors::asset::{AssetDynamoDBError, AssetNoExistsError},
    services::assets::{AssetManipulation, AssetService},
};
use serde_json::json;
use tracing::info;
use url::Url;
use uuid::Uuid;
use validator::ValidationError;

use crate::my_lambda::{build_resp, build_resp_env, build_resp_no_cache};

//#[instrument]
pub async fn get_similar_assets_by_id(
    _req: &Request,
    _c: &Context,
    config: &Config,
    asset_service: &AssetService,
    video_service: &VideoService,
    asset_id: &Uuid,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let op_res = asset_service.get_by_id(asset_id).await;
    match op_res {
        Ok(_asset) => {
            info!("requesting similar ones for: {}", asset_id);

            let res = video_service.get_similar_hashes(asset_id).await?;

            info!("completed!");
            build_resp_no_cache(json!(res).to_string(), StatusCode::OK)
        }
        Err(e) => {
            if let Some(e) = e.downcast_ref::<AssetDynamoDBError>() {
                return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE);
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
    }
}

#[derive(serde::Deserialize, Clone, Debug )]
struct UrlSimilar {
    url: Url,
}

pub async fn get_similar_assets_by_url(
    req: &Request,
    _c: &Context,
    config: &Config,
    asset_service: &AssetService,
    video_service: &VideoService,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {

    let body_str = std::str::from_utf8(req.body().as_ref())?;
    let event: serde_json::Value = serde_json::from_str(body_str)?;

    let url_from_query = event["queryStringParameters"]["url"].as_str().map(str::to_string);

    let url_asset = if let Some(url) = url_from_query {
        let aux = Url::parse(&url)?;
        UrlSimilar { url: aux } 
    } else {
        match req.payload::<UrlSimilar>() {
            Err(e) => {
                return build_resp(e.to_string(), StatusCode::BAD_REQUEST);
            }
            Ok(Some(payload)) => payload,
            Ok(None) => {
                return build_resp("no URL found".to_string(), StatusCode::BAD_REQUEST);
            }
        }
    };

    let op_res = asset_service.get_by_url(&url_asset.url).await;
    match op_res {
        Ok(asset) => {
            info!("requesting similar ones for: {}", asset.id());

            let res = video_service.get_similar_hashes(asset.id()).await?;

            info!("completed!");
            build_resp_no_cache(json!(res).to_string(), StatusCode::OK)
        }
        Err(e) => {
            if let Some(e) = e.downcast_ref::<AssetDynamoDBError>() {
                return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE);
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
    }
}
