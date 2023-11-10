pub mod my_lambda;

use std::str::FromStr;

use lib_config::config::Config;
use lib_engage::{services::subscription::SubscriptionService, repositories::subscription::SubscriptionRepo};
use lib_licenses::services::{assets::AssetService, owners::OwnerService, video::VideoService};
use lib_users::services::users::UsersService;
use lambda_http::{http::Method, http::StatusCode, IntoResponse, Request, RequestExt};
use matchit::Router;
use url::Url;
use uuid::Uuid;

use crate::my_lambda::{build_resp, assets::{get_asset::{get_asset_by_url, get_asset_by_id}, get_similar_assets::{get_similar_assets_by_id, get_similar_assets_by_url}, create_asset::create_asset}, jwt_mandatory, video::async_create_my_hash::async_create_my_hash_similars_sns, subscribe::subscribe::{create_intent, confirm_subscription, remove_subscription}};



//#[tracing::instrument]
pub async fn function_handler(
    config: &Config,
    asset_service: &AssetService,
    owners_service: &OwnerService,
    user_service: &UsersService,
    video_service: &VideoService,
    //_license_service: &LicenseService,
    subscription_service: &SubscriptionService<SubscriptionRepo>,
    req: Request,
) -> Result<impl IntoResponse, Box<dyn std::error::Error + Send + Sync>> {
    log::info!("income new request");
    let context = req.lambda_context();
    let user_id;

    let mut router = Router::new();
    router.insert("/api/asset", Some("1"))?;
    router.insert("/api/asset/:id", Some("2"))?;
    router.insert("/api/hash", Some("88"))?;
    router.insert("/api/similar/:id", Some("99"))?;
    router.insert("/api/similar", Some("999"))?;
    router.insert("/api/subscribe", Some("1000"))?;
    router.insert("/api/subscribe/confirmation/:id", Some("1001"))?;
    router.insert("/api/subscribe/remove/:id", Some("1002"))?;

    let query_pairs: Vec<(String, String)> = req.uri().query()
            .map(|v| url::form_urlencoded::parse(v.as_bytes()).into_owned().collect())
            .unwrap_or_else(Vec::new);

    match req.method() {
        &Method::GET => match router.at(req.uri().path()) {
            Err(_) => build_resp(
                "method not allowed - ".to_string(),
                StatusCode::METHOD_NOT_ALLOWED,
            ),
            Ok(matched) => match matched.value.unwrap() {
                "1" => {

                    let id_opt = query_pairs
                        .iter()
                        .find(|(key, _)| key == "url")
                        .map(|(_, value)| value.clone());

                    if let Some(id) = id_opt {
                        let url = Url::from_str(&id)?;
                        get_asset_by_url(
                            &req,
                            &context,
                            config,
                            asset_service,
                            &url,
                        )
                        .await
                    } else {
                        // Handle the case where the id parameter is not present in the query string
                        // For instance, you can return an error response:
                        build_resp(
                            "url not found in query string".to_string(),
                            StatusCode::BAD_REQUEST,
                        )
                    }
                }
                "2" => {
                    // public, not required jwt token
                    let id = matched.params.get("id").unwrap().to_string();
                    let asset_id = Uuid::from_str(id.as_str())?;
                    return get_asset_by_id(
                        &req,
                        &context,
                        config,
                        asset_service,
                        owners_service,
                        &asset_id,
                    )
                    .await;
                }
                
                "99" => {
                    let id = matched.params.get("id").unwrap().to_string();

                    if let Ok(asset_id) = Uuid::from_str(id.as_str()) {
                        return get_similar_assets_by_id(
                            &req,
                            &context,
                            config,
                            asset_service,
                            video_service,
                            &asset_id,
                        )
                        .await;
                    } else {
                        build_resp(
                            "id param must be UUID".to_string(),
                            StatusCode::NOT_ACCEPTABLE,
                        )
                    }
                }

                "999" => {
                    //let url = matched.params.get("url").unwrap().to_string();
                    return get_similar_assets_by_url(
                        &req,
                        &context,
                        config,
                        asset_service,
                        video_service,
                    )
                    .await;
                }

                _ => build_resp(
                    "GET method not allowed".to_string(),
                    StatusCode::METHOD_NOT_ALLOWED,
                ),
            },
        },
        &Method::POST => match router.at(req.uri().path()) {
            Err(_) => build_resp(
                "method not allowed *".to_string(),
                StatusCode::METHOD_NOT_ALLOWED,
            ),
            Ok(matched) => match matched.value.unwrap() {
                "1" => {
                    let ussrr = match jwt_mandatory(&req, config) {
                        Err(_) => None,
                        Ok(user) => Some(user),
                    };
                    create_asset(&req, &context, config, asset_service, video_service, ussrr).await
                }
                
                "88" => {
                    match jwt_mandatory(&req, config) {
                        Err(e) => {
                            return Ok(e);
                        }
                        Ok(user) => user_id = user,
                    };

                    return async_create_my_hash_similars_sns(
                        &req,
                        &context,
                        config,
                        asset_service,
                        video_service,
                        &user_id,
                    )
                    .await;
                }

                "999" => {
                    return get_similar_assets_by_url(
                        &req,
                        &context,
                        config,
                        asset_service,
                        video_service,
                    )
                    .await;
                }

                "1000" => {
                    return create_intent(
                        &req,
                        &context,
                        config,
                        asset_service,
                        video_service,
                        user_service,
                        subscription_service,
                    )
                    .await;
                }

                "1001" => {
                    let id = matched.params.get("id").unwrap().to_string();
                    if let Ok(subscription_id) = Uuid::from_str(id.as_str()) {
                        return confirm_subscription(
                            &req,
                            &context,
                            config,
                            subscription_service,
                            subscription_id,
                        )
                        .await;
                    } else {
                        build_resp(
                            "id param must be UUID".to_string(),
                            StatusCode::NOT_ACCEPTABLE,
                        )
                    }
                }

                "1002" => {
                    let id = matched.params.get("id").unwrap().to_string();
                    if let Ok(subscription_id) = Uuid::from_str(id.as_str()) {
                        return remove_subscription(
                            &req,
                            &context,
                            config,
                            subscription_service,
                            subscription_id,
                        )
                        .await;
                    } else {
                        build_resp(
                            "id param must be UUID".to_string(),
                            StatusCode::NOT_ACCEPTABLE,
                        )
                    }
                }
                &_ => build_resp(
                    "POST method not allowed".to_string(),
                    StatusCode::METHOD_NOT_ALLOWED,
                ),
            },
        },
        _ => build_resp(
            "http verb doesn't use it here".to_string(),
            StatusCode::METHOD_NOT_ALLOWED,
        ),
    }
}

