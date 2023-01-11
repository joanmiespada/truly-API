use lambda_http::{
      http::StatusCode, lambda_runtime::Context,
      Request, RequestExt, Response,
};
use lib_users::errors::users::{DynamoDBError, UserMismatchError, UserAlreadyExistsError};
use lib_users::models::user::User;
use serde::{Deserialize, Serialize };
use serde_json::json;
use tracing::{instrument};
use lib_config::Config;
use lib_users::services::users::{UsersService, UserManipulation};

use crate::my_lambda::{build_resp };

#[derive(Debug,Deserialize)]
pub struct NewUser {
    pub wallet_address: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub device: Option<String>,
}

#[derive(Debug,Serialize)]
pub struct NewIdUser {
    pub id: String
}
#[instrument]
pub async fn create_basic_user (
    _req: &Request,
    _c: &Context,
    config: &Config,
    user_service: &UsersService,
) -> Result<Response<String>, Box<dyn std::error::Error>> {
 

    let mut user = User::new();
    let new_password;
    match _req.payload::<NewUser>() {
        Err(e)=> { return build_resp(e.to_string(), StatusCode::BAD_REQUEST);},
        Ok(op_user)=> match op_user {
            None => { return build_resp("no user data found!".to_string(), StatusCode::BAD_REQUEST); },
            Some(payload) => {
                if let Some(eml) = &payload.email{
                    user.set_email( eml );
                }
                if let Some(wll) = &payload.wallet_address {
                    user.set_wallet_address(wll);
                }
                if let Some(dvc) = &payload.device {
                    user.set_device(dvc);
                }
                new_password = payload.password
            }
        } 
    }

    let op_res = user_service.add_user(&mut user, &new_password).await;
    match op_res {
        Err(e) => {
            if let Some(err) = e.downcast_ref::<DynamoDBError>() {
                //HttpResponse::ServiceUnavailable().body(err.to_string())    
                build_resp(err.to_string(), StatusCode::SERVICE_UNAVAILABLE)
            } else if  let Some(err) = e.downcast_ref::<UserAlreadyExistsError>() {
                //HttpResponse::BadRequest().body(err.to_string())
                build_resp(err.to_string(), StatusCode::NOT_ACCEPTABLE)
            } else if  let Some(err) = e.downcast_ref::<UserMismatchError>() {
                //HttpResponse::BadRequest().body( err.to_string())
                build_resp(err.to_string(), StatusCode::NOT_ACCEPTABLE)
            } else {
                //HttpResponse::InternalServerError().finish()    
                build_resp(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
        Ok(iid) => { 
            let res= NewIdUser{ id: iid};
            //HttpResponse::Ok().json(res)
            build_resp( json!(res).to_string(), StatusCode::OK)
        }
    }

}