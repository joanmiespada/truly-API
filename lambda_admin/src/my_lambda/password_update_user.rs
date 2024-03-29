use super::build_resp;
use lambda_http::RequestPayloadExt;
use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::config::Config;
use lib_config::result::ResultE;
use lib_users::errors::users::{UserDynamoDBError, UserNoExistsError};
use lib_users::services::users::{UserManipulation, UsersService};
use serde::{Deserialize, Serialize};

use lib_users::validate_password;
use validator::{Validate, ValidationError};
#[derive(Debug, Serialize, Validate, Deserialize)]
pub struct UpdatePasswordUser {
    #[validate(length(min = 8, max = 50), custom = "validate_password")]
    pub password: String,
}

//#[instrument]
pub async fn password_update_user(
    req: &Request,
    _c: &Context,
    _config: &Config,
    user_service: &UsersService,
    id: &String,
) -> ResultE<Response<String>> {
    let new_password;
    match req.payload::<UpdatePasswordUser>() {
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
                    new_password = payload.password.clone();
                }
            },
        },
    }

    let op_res = user_service.update_password(id, &new_password).await;
    match op_res {
        Err(e) => {
            if let Some(m) = e.downcast_ref::<UserDynamoDBError>() {
                return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
                //HttpResponse::ServiceUnavailable().finish()
            } else if let Some(m) = e.downcast_ref::<UserNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
                //HttpResponse::BadRequest().finish()
            } else if let Some(m) = e.downcast_ref::<ValidationError>() {
                return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
            } else {
                return build_resp(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR);
                //HttpResponse::InternalServerError().finish()
            }
        }
        Ok(_) => {
            build_resp("".to_string(), StatusCode::OK)
            //HttpResponse::Ok().finish()
        }
    }
}
