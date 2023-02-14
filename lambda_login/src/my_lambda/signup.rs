use crate::my_lambda::build_resp;
use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, RequestExt, Response};
use lib_config::config::Config;
use lib_users::errors::users::{UserDynamoDBError, UserAlreadyExistsError, UserMismatchError};
use lib_users::models::user::User;
use lib_users::services::users::{UserManipulation, UsersService};
use lib_users::validate_password;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::instrument;
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate, Default)]
pub struct NewUser {
    pub wallet_address: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 8, max = 50), custom = "validate_password")]
    pub password: Option<String>,
    pub device: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NewIdUser {
    pub id: String,
}
#[instrument]
pub async fn create_basic_user(
    _req: &Request,
    _c: &Context,
    config: &Config,
    user_service: &UsersService,
) -> Result<Response<String>, Box<dyn std::error::Error>> {

    let mut user = User::new();
    let new_password;
    match _req.payload::<NewUser>() {
        Err(e) => {
            return build_resp(e.to_string(), StatusCode::BAD_REQUEST);
        }
        Ok(op_user) => match op_user {
            None => {
                return build_resp("no user data found!".to_string(), StatusCode::BAD_REQUEST);
            }
            Some(payload) => match payload.validate() {
                Err(e) => {
                    return build_resp(e.to_string(), StatusCode::BAD_REQUEST);
                }
                Ok(_) => {
                    if let Some(eml) = &payload.email {
                        user.set_email(eml);
                    }
                    if let Some(dvc) = &payload.device {
                        user.set_device(dvc);
                    }
                    new_password = payload.password
                }
            },
        },
    }

    let op_res = user_service.add_user(&mut user, &new_password).await;
    match op_res {
        Err(e) => {
            if let Some(err) = e.downcast_ref::<UserDynamoDBError>() {
                //HttpResponse::ServiceUnavailable().body(err.to_string())
                build_resp(err.to_string(), StatusCode::SERVICE_UNAVAILABLE)
            } else if let Some(err) = e.downcast_ref::<UserAlreadyExistsError>() {
                //HttpResponse::BadRequest().body(err.to_string())
                build_resp(err.to_string(), StatusCode::NOT_ACCEPTABLE)
            } else if let Some(err) = e.downcast_ref::<UserMismatchError>() {
                //HttpResponse::BadRequest().body( err.to_string())
                build_resp(err.to_string(), StatusCode::NOT_ACCEPTABLE)
            }else if let Some(m) = e.downcast_ref::<ValidationError>() {
                return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
            }else {
                //HttpResponse::InternalServerError().finish()
                build_resp(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
        Ok(iid) => {
            let res = NewIdUser { id: iid };
            //HttpResponse::Ok().json(res)
            build_resp(json!(res).to_string(), StatusCode::OK)
        }
    }
}
