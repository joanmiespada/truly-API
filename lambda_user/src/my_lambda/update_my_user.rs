
use lambda_http::RequestExt;
use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::Config;
use lib_users::errors::users::{DynamoDBError, UserNoExistsError};
use lib_users::services::users::{UserManipulation, UsersService, UpdatableFildsUser};
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUser {
    pub wallet_address: Option<String>,
    pub device: Option<String>,
    pub email: Option<String>,
    pub status: Option<String>,
}
use super::build_resp;
#[instrument]
pub async fn update_my_user(
    req: &Request,
    _c: &Context,
    config: &Config,
    user_service: &UsersService,
    id: &String,
) -> Result<Response<String>, Box<dyn std::error::Error>> {

    
    let user_fields;
    match req.payload::<UpdateUser>() {
        Err(e) => {
            return build_resp(e.to_string(), StatusCode::BAD_REQUEST);
        }
        Ok(op_payload) => match op_payload {
            None => {
                return build_resp("no payload found".to_string(), StatusCode::BAD_REQUEST);
            }
            Some(payload) => {
                user_fields = UpdatableFildsUser {
                    device: payload.device.clone(),
                    email: payload.email.clone(),
                    wallet_address: payload.wallet_address.clone(),
                    status: payload.status.clone(),
                };
            }
        },
    }

    let op_res = user_service.update_user(&id, &user_fields).await;
    match op_res {
        Err(e) => {
            if let Some(m) = e.downcast_ref::<DynamoDBError>() {
                return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<UserNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else {
                return build_resp("".to_string(), StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
        Ok(_) => {
            build_resp("".to_string(), StatusCode::ACCEPTED)
        }
    }
}
