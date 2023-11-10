use crate::my_lambda::assets::create_asset::create_asset;
use crate::my_lambda::{build_resp, build_resp_env};
use lambda_http::RequestPayloadExt;
use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::config::Config;
use lib_engage::errors::subscription::SubscriptionError;
use lib_engage::repositories::subscription::SubscriptionRepo;
use lib_engage::services::subscription::SubscriptionService;
use lib_licenses::errors::asset::{AssetDynamoDBError, AssetNoExistsError};
use lib_licenses::services::assets::{AssetManipulation, AssetService};
use lib_licenses::services::video::VideoService;
use lib_users::errors::users::{UserDynamoDBError, UserNoExistsError};
use lib_users::models::user::User;
use lib_users::services::users::{UserManipulation, UsersService};
use log::info;
use serde::{Deserialize, Serialize};
use url::Url;
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct CreatableFildsSubscription {
    #[validate(length(max = 1000))]
    pub email: String,

    pub url: Url,
}

//#[instrument]
pub async fn create_intent(
    req: &Request,
    c: &Context,
    config: &Config,
    asset_service: &AssetService,
    video_service: &VideoService,
    user_service: &UsersService,
    subscription_service: &SubscriptionService<SubscriptionRepo>,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let subscription_fields;
    match req.payload::<CreatableFildsSubscription>() {
        Err(e) => {
            return build_resp(e.to_string(), StatusCode::BAD_REQUEST);
        }
        Ok(op_payload) => match op_payload {
            None => {
                return build_resp("no payload found".to_string(), StatusCode::BAD_REQUEST);
            }
            Some(payload) => subscription_fields = payload.clone(),
        },
    }
    log::info!("subscription_fields: {:?}", subscription_fields);
    let asset;
    let asset_op = asset_service.get_by_url(&subscription_fields.url).await;
    if let Err(e) = asset_op {
        if let Some(m) = e.downcast_ref::<AssetDynamoDBError>() {
            return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
        } else if let Some(_) = e.downcast_ref::<AssetNoExistsError>() {
            create_asset(req, c, config, asset_service, video_service, None).await?;
            asset = asset_service.get_by_url(&subscription_fields.url).await?;
            log::info!("asset not found, but created successfully: {:?}", asset);
        } else if let Some(m) = e.downcast_ref::<ValidationError>() {
            return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
        } else {
            return build_resp_env(
                &config.env_vars().environment().unwrap(),
                e,
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    } else {
        asset = asset_op.unwrap();
        log::info!("asset found!: {:?}", asset);
    }

    let user;
    let user_op = user_service.get_by_email(&subscription_fields.email).await;
    if let Err(e) = user_op {
        if let Some(m) = e.downcast_ref::<UserDynamoDBError>() {
            return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
        } else if let Some(_) = e.downcast_ref::<UserNoExistsError>() {
            let mut user1 = User::new();
            user1.set_email(&subscription_fields.email);
            let user_id = user_service.add(&mut user1, &None).await?;
            user = user_service.get_by_id(&user_id).await?;
            log::info!("user not found, but created successfully: {:?}", user);
        //} else if let Some(_) = e.downcast_ref::<UserAlreadyExistsError>() {
        } else if let Some(m) = e.downcast_ref::<ValidationError>() {
            return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
        } else {
            return build_resp_env(
                &config.env_vars().environment().unwrap(),
                e,
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    } else {
        user = user_op.unwrap();
        log::info!("user found!: {:?}", user);
    }

    info!("calling subscription service: intent");
    let op1 = subscription_service.intent(user, asset).await;

    if let Err(e) = op1 {
        if let Some(err_m) = e.downcast_ref::<SubscriptionError>() {
            match err_m {
                SubscriptionError::SubscriptionDynamoDBError(_) => {
                    return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE)
                }
                _ => return build_resp(e.to_string(), StatusCode::NOT_ACCEPTABLE),
            }
            //return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
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
    let subscription_id = op1.unwrap().clone();
    log::info!("subscription_id completed successfully: {}", subscription_id.to_string());

    build_resp(subscription_id.to_string(), StatusCode::OK)
}

pub async fn confirm_subscription(
    _req: &Request,
    _c: &Context,
    config: &Config,
    subscription_service: &SubscriptionService<SubscriptionRepo>,
    id: uuid::Uuid,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let op1 = subscription_service.confirm(id).await;

    if let Err(e) = op1 {
        if let Some(err_m) = e.downcast_ref::<SubscriptionError>() {
            match err_m {
                SubscriptionError::SubscriptionDynamoDBError(_) => {
                    return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE)
                }
                SubscriptionError::SubscriptionIDNotFound(_) => {
                    return build_resp(e.to_string(), StatusCode::NOT_FOUND)
                }
                _ => return build_resp(e.to_string(), StatusCode::NOT_ACCEPTABLE),
            }
        } else {
            return build_resp_env(
                &config.env_vars().environment().unwrap(),
                e,
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    }

    build_resp("".to_string(), StatusCode::OK)
}

pub async fn remove_subscription(
    _req: &Request,
    _c: &Context,
    config: &Config,
    subscription_service: &SubscriptionService<SubscriptionRepo>,
    id: uuid::Uuid,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let op1 = subscription_service.delete(id).await;

    if let Err(e) = op1 {
        if let Some(err_m) = e.downcast_ref::<SubscriptionError>() {
            match err_m {
                SubscriptionError::SubscriptionDynamoDBError(_) => {
                    return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE)
                }
                SubscriptionError::SubscriptionIDNotFound(_) => {
                    return build_resp(e.to_string(), StatusCode::NOT_FOUND)
                }
                _ => return build_resp(e.to_string(), StatusCode::NOT_ACCEPTABLE),
            }
        } else {
            return build_resp_env(
                &config.env_vars().environment().unwrap(),
                e,
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    }

    build_resp("".to_string(), StatusCode::OK)
}
