
use lambda_http::{
      http::StatusCode, lambda_runtime::Context,
      Request, RequestExt, Response,
};
use lib_users::errors::users::{UserDynamoDBError, UserNoExistsError, UserStatusError};
use lib_users::services::login::LoginOps;
use lib_util_jwt::create_jwt;
use serde::Deserialize;
use serde_json::json;
use tracing::instrument;
use lib_config::Config;
use lib_users::services::users::UsersService;

use crate::my_lambda::build_resp;
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate )]
pub struct LoginPayload {
    #[serde(default)]
    #[validate(email)]
    pub email: Option<String>,
    #[serde(default)]
    #[validate(length(min = 8, max=50))]
    pub password: Option<String>,
    #[serde(default)]
    pub device: Option<String>,
}



#[instrument]
pub async fn login(
    _req: &Request,
    _c: &Context,
    config: &Config,
    user_service: &UsersService,
) -> Result<Response<String>, Box<dyn std::error::Error>> {
    //let method_name = event.into_parts().0;
    let args = _req.payload::<LoginPayload>();
    
     match args{
         Err(_) => build_resp("no correct payload attached to the request: either username and password, or device, are mandatories".to_string(), StatusCode::BAD_REQUEST ),
         Ok(user_pass) => {
             match user_pass {
                 None => build_resp("either email/password or device fields are empty".to_string(), StatusCode::BAD_REQUEST),
                 Some(payload) => {
                    match payload.validate(){
                        Err(e) => { return build_resp(e.to_string(), StatusCode::BAD_REQUEST); }
                        Ok(_) => {}
                    }
                    let result = user_service.login(&payload.device, &payload.email, &payload.password).await;
                    match result {
                        Err(e) => {
                            if let Some(_) = e.downcast_ref::<UserDynamoDBError>() {
                                build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE)
                            } else if let Some(e) = e.downcast_ref::<UserNoExistsError>() {
                                build_resp(e.to_string(), StatusCode::NOT_ACCEPTABLE)
                            } else if let Some(m) = e.downcast_ref::<ValidationError>() {
                                build_resp(m.to_string(), StatusCode::BAD_REQUEST)
                            }else if let Some(m) = e.downcast_ref::<UserStatusError>() {
                                build_resp(m.to_string(), StatusCode::FORBIDDEN)
                            }else {
                                build_resp(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
                            }
                        }
                        Ok(log_inf) => {
                            
                            let roles: Vec<String> = log_inf.roles.into_iter().map(|ur| ur.to_string()).collect();
                            let token_secret = config.env_vars().jwt_token_base();
                            let exp_hours =  config.env_vars().jwt_token_time_exp_hours().parse::<i64>().unwrap();
                            let token_creation_ops = create_jwt(&log_inf.user_id, roles, token_secret, exp_hours );
            
                            match token_creation_ops {
                                Err(_) => build_resp("".to_string(), StatusCode::INTERNAL_SERVER_ERROR),
                                Ok(token) => build_resp (  json!({ "token": token }).to_string()  ,StatusCode::OK)
                                
                            }
                            
                        }
                    }
                 }
             }
         }
     }
}


