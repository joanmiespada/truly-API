use crate::my_lambda::assets::create_asset::create_asset;
use crate::my_lambda::{build_resp, build_resp_env};
use lambda_http::RequestPayloadExt;
use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::config::Config;
use lib_licenses::errors::asset::{AssetDynamoDBError, AssetNoExistsError};
use lib_licenses::errors::subscription::SubscriptionError;
use lib_licenses::services::assets::{AssetManipulation, AssetService };
use lib_licenses::services::video::VideoService;
use lib_users::errors::users::UserNoExistsError;
use lib_users::models::user::User;
use serde::{Deserialize,Serialize};
use tracing::info;
use url::Url;
use validator::{ValidationError, Validate};
use lib_users::services::users::{UsersService,UserManipulation};
use lib_licenses::services::subscription::SubscriptionService;


#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct CreatableFildsSubscription {
    #[validate(length(max = 1000))]
    pub email: String,

    pub url: Url
}


//#[instrument]
pub async fn create_intent(
    req: &Request,
    c: &Context,
    config: &Config,
    asset_service: &AssetService,
    video_service: &VideoService,
    user_service: &UsersService,
    subscription_service: &SubscriptionService,
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
    let asset_id;
    let asset_op = asset_service.get_by_url(&subscription_fields.url).await;
    if let Err(e) = asset_op{
        if let Some(m) = e.downcast_ref::<AssetDynamoDBError>() {
            return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
        } else if let Some(_) = e.downcast_ref::<AssetNoExistsError>() {

            create_asset(req, c, config, asset_service, video_service, None).await?;
            asset_id = asset_service.get_by_url(&subscription_fields.url).await?.id().clone();

        } else if let Some(m) = e.downcast_ref::<ValidationError>() {
            return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
        } else {
            return build_resp_env(
                &config.env_vars().environment().unwrap(),
                e,
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    }else{
        asset_id = asset_op.unwrap().id().clone();
    }
    

    let user_id;
    let user_op = user_service.get_by_email(&subscription_fields.email).await;
    if let Err(e) = user_op{
        if let Some(m) = e.downcast_ref::<AssetDynamoDBError>() {
            return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
        } else if let Some(_) = e.downcast_ref::<UserNoExistsError>() {

            let mut user = User::new();
            user.set_email(&subscription_fields.email);
            user_id = user_service.add(&mut user, &None).await?;

        } else if let Some(m) = e.downcast_ref::<ValidationError>() {
            return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
        } else {
            return build_resp_env(
                &config.env_vars().environment().unwrap(),
                e,
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    }else{
        user_id = user_op.unwrap().user_id().clone();
    }


    info!("calling asset service: add");
    let op1 = subscription_service.intent(user_id,asset_id).await;

    if let Err(e) = op1 {
        if let Some(err_m) = e.downcast_ref::<SubscriptionError>() {
            match err_m{
                SubscriptionError::SubscriptionDynamoDBError(_) => return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE),
                _ => return build_resp(e.to_string(), StatusCode::NOT_ACCEPTABLE)
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

    build_resp("".to_string(), StatusCode::OK)

}

pub async fn confirm_subscription(
    _req: &Request,
    _c: &Context,
    config: &Config,
    subscription_service: &SubscriptionService,
    id: uuid::Uuid,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    
    let op1 = subscription_service.confirm(id).await;

    if let Err(e) = op1 {
        if let Some(err_m) = e.downcast_ref::<SubscriptionError>() {

            match err_m {
                SubscriptionError::SubscriptionDynamoDBError(_) => return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE),
                SubscriptionError::SubscriptionIDNotFound(_) => return build_resp(e.to_string(), StatusCode::NOT_FOUND),
                _ => return build_resp(e.to_string(), StatusCode::NOT_ACCEPTABLE)
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
