use lambda_http::RequestExt;
use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::config::Config;
use lib_users::errors::users::{UserDynamoDBError, UserNoExistsError};
use lib_users::services::users::{UpdatableFildsUser, UserManipulation, UsersService};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use validator::{Validate,ValidationError};

#[derive(Debug, Serialize, Validate, Deserialize)]
pub struct UpdateUser {
    pub device: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    //pub status: Option<String>, //forbidden, only by admins. Roles idem, only admin can change it 
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
            Some(payload) => match payload.validate() {
                Err(e) => {
                    return build_resp(e.to_string(), StatusCode::BAD_REQUEST);
                }
                Ok(_) => {
                    user_fields = UpdatableFildsUser {
                        device: payload.device.clone(),
                        email: payload.email.clone(),
                        wallet: None,
                        status: None
                        //status: payload.status.clone(), //forbidden here, only at admins lambda
                    };
                }
            },
        },
    }

    let op_res = user_service.update(&id, &user_fields).await;

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
