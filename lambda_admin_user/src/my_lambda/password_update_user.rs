use lambda_http::RequestExt;
use lambda_http::{
      http::StatusCode, lambda_runtime::Context,
       Request,  Response,
};
use lib_users::errors::users::{DynamoDBError, UserNoExistsError};
use lib_users::services::users::{UsersService, UserManipulation};
use tracing::{instrument};
use lib_config::Config;
use serde::{Deserialize ,Serialize };
use super::build_resp;

#[derive(Debug,Serialize,Deserialize)]
pub struct UpdatePasswordUser {
    pub password: String, 
}

#[instrument]
pub async fn password_update_user (
    req: &Request,
    _c: &Context,
    config: &Config,
    user_service: &UsersService,
) -> Result<Response<String>, Box<dyn std::error::Error>> {

    let id;
    
    match req.path_parameters().all("id") {
        None => { return build_resp("no id specified on url path".to_string(), StatusCode::BAD_REQUEST);},
        Some(vstr) => {
            let _first = vstr.first();
            match _first {
                None => { return build_resp("no id specified on url path".to_string(), StatusCode::BAD_REQUEST);},
                Some(value) => id = value.to_string()
            }
        }
    }

    let new_password;
    match req.payload::<UpdatePasswordUser>() {
        Err(e) => {
            return build_resp(e.to_string(), StatusCode::BAD_REQUEST);
        }
        Ok(op_payload) => match op_payload {
            None => {
                return build_resp("no payload found".to_string(), StatusCode::BAD_REQUEST);
            }
            Some(payload) => {
                new_password = payload.password.clone();
            }
        },
    }

    let op_res = user_service. update_password(&id, &new_password).await;
    match op_res {
        Err(e) => {
            if let Some(m) = e.downcast_ref::<DynamoDBError>() {
                return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
                //HttpResponse::ServiceUnavailable().finish()    
            } else if  let Some(m) = e.downcast_ref::<UserNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
                //HttpResponse::BadRequest().finish()
            } else {
                return build_resp(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR);
                //HttpResponse::InternalServerError().finish()    
            }
        },
        Ok(_) => { 
            build_resp( "".to_string() , StatusCode::ACCEPTED  )
            //HttpResponse::Ok().finish()
        }
    }
}