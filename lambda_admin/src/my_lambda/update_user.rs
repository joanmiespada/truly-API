use lambda_http::RequestPayloadExt;
use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::config::Config;
use lib_config::result::ResultE;
use lib_users::errors::users::{UserDynamoDBError, UserNoExistsError};
use lib_users::services::users::{UpdatableFildsUser, UserManipulation, UsersService};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, Validate, Deserialize)]
pub struct UpdateUser {
    pub wallet_address: Option<String>,
    pub device: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub status: Option<String>,
}
use super::build_resp;

//#[instrument]
pub async fn update_user(
    req: &Request,
    _c: &Context,
    _config: &Config,
    user_service: &UsersService,
    id: &String,
) -> ResultE<Response<String>> {
    let user_fields;
    match req.payload::<UpdateUser>() {
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
                    user_fields = UpdatableFildsUser {
                        device: payload.device.clone(),
                        email: payload.email.clone(),
                        status: payload.status.clone(),
                        wallet: None,
                    };
                }
            },
        },
    }

    let op_res = user_service.update(id, &user_fields).await;
    match op_res {
        Err(e) => {
            if let Some(m) = e.downcast_ref::<UserDynamoDBError>() {
                return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<UserNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else if let Some(m) = e.downcast_ref::<ValidationError>() {
                return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
            } else {
                return build_resp("".to_string(), StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
        Ok(_) => build_resp("".to_string(), StatusCode::OK),
    }
}
