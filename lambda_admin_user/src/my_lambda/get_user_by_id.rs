use lambda_http::RequestExt;
use lambda_http::{
      http::StatusCode, lambda_runtime::Context,
       Request,  Response,
};
use lib_users::errors::users::{DynamoDBError, UserNoExistsError};
use lib_users::services::users::{UsersService, UserManipulation};
use serde_json::json;
use tracing::{instrument};
use lib_config::Config;

use super::build_resp;


#[instrument]
pub async fn get_user_by_id(
    req: &Request,
    _c: &Context,
    config: &Config,
    user_service: &UsersService,
    id: &String
) -> Result<Response<String>, Box<dyn std::error::Error>> {
 
    /*let id;
    
    match req.path_parameters().all("id") {
        None => { return build_resp("no id specified on url path".to_string(), StatusCode::BAD_REQUEST);},
        Some(vstr) => {
            let _first = vstr.first();
            match _first {
                None => { return build_resp("no id specified on url path".to_string(), StatusCode::BAD_REQUEST);},
                Some(value) => id = value.to_string()
            }
        }
    }*/


    let op_res = user_service.get_by_user_id(id).await;
    match op_res {
        Ok(user) => build_resp( json!(user).to_string()  , StatusCode::ACCEPTED  ),
        Err(e) => {
            if let Some(e) = e.downcast_ref::<DynamoDBError>() {
                return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if  let Some(m) = e.downcast_ref::< UserNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
                //HttpResponse::NoContent().finish()
            } else {
                return build_resp(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR);
                //HttpResponse::InternalServerError().finish()    
            }
        }
    }
    
}