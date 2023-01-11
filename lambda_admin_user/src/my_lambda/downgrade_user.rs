use lambda_http::RequestExt;
use lambda_http::{
      http::StatusCode, lambda_runtime::Context,
       Request,  Response,
};
use lib_users::errors::users::{DynamoDBError, UserNoExistsError};
use lib_users::services::users::{UsersService, UserManipulation, PromoteUser};
use tracing::{instrument};
use lib_config::Config;

use super::build_resp;

#[instrument]
pub async fn downgrade_user (
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

    let op_res = user_service.promote_user_to(&id, &PromoteUser::Downgrade ).await;
    match op_res {
        Err(e) => {
            if let Some(e) = e.downcast_ref::<DynamoDBError>() {
                return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE);
                //HttpResponse::ServiceUnavailable().finish()    
            } else if  let Some(e) = e.downcast_ref::<UserNoExistsError>() {
                return build_resp(e.to_string(), StatusCode::NO_CONTENT);
                //HttpResponse::BadRequest().finish()
            } else {
                return build_resp("".to_string(), StatusCode::INTERNAL_SERVER_ERROR);
                //HttpResponse::InternalServerError().finish()    
            }
        },
        Ok(_) => { 
            build_resp( "".to_string() , StatusCode::ACCEPTED  )
            //HttpResponse::Ok().body(iid.to_string())
        }
    }
}

