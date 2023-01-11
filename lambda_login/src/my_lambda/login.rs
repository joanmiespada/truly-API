
use lambda_http::{
      http::StatusCode, lambda_runtime::Context,
      Request, RequestExt, Response,
};
use lib_users::errors::users::{DynamoDBError, UserNoExistsError};
use lib_users::services::login::LoginOps;
use lib_util_jwt::create_jwt;
use serde::{Deserialize, Serialize };
use serde_json::json;
use tracing::{instrument};
use lib_config::Config;
use lib_users::services::users::UsersService;

use crate::my_lambda::{build_resp };


#[derive(Debug, Deserialize,Serialize, Default)]
pub struct LoginPayload {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
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

    println!("running the login...");
    
     match args{
         Err(_) => build_resp("no correct payload attached to the request: either username and password, or device, are mandatories".to_string(), StatusCode::BAD_REQUEST ),
         Ok(user_pass) => {
             match user_pass {
                 None => build_resp("either email/password or device fields are empty".to_string(), StatusCode::BAD_REQUEST),
                 Some(payload) => {
                    println!("payload received for login");
                    let result = user_service.login(&payload.device, &payload.email, &payload.password).await;
                    println!("login service called");
                    match result {
                        Err(e) => {
                            if let Some(_) = e.downcast_ref::<DynamoDBError>() {
                                println!("error: {}",e.to_string());
                                build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE)
                            } else if let Some(e) = e.downcast_ref::<UserNoExistsError>() {
                                let aux2 = serde_json::to_string(&payload).unwrap();
                                let aux1 = format!("error: {} --payload received: {}",e.to_string(), aux2 );
                                println!("{}",aux1);
                                build_resp(aux1, StatusCode::NOT_ACCEPTABLE)
                            } else {
                                println!("error: {}",e.to_string());
                                build_resp(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
                            }
                        }
                        Ok(log_inf) => {
                            
                            let roles: Vec<String> = log_inf.roles.into_iter().map(|ur| ur.to_string()).collect();
                            let token_secret = config.env_vars().jwt_token_base();
                            let token_creation_ops = create_jwt(&log_inf.user_id, &roles, token_secret );
            
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

