use lambda_http::{
      http::StatusCode, lambda_runtime::Context,
       Request,  Response,
};
use lib_users::errors::users::{DynamoDBError, UserNoExistsError};
use lib_users::services::users::{UsersService, UserManipulation, PromoteUser};
use tracing::instrument;
use lib_config::Config;

use super::build_resp;

#[instrument]
pub async fn downgrade_user (
    req: &Request,
    _c: &Context,
    config: &Config,
    user_service: &UsersService,
    id: &String
) -> Result<Response<String>, Box<dyn std::error::Error>> {

    let op_res = user_service.promote_user_to(id, &PromoteUser::Downgrade ).await;
    match op_res {
        Err(e) => {
            if let Some(e) = e.downcast_ref::<DynamoDBError>() {
                return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if  let Some(e) = e.downcast_ref::<UserNoExistsError>() {
                return build_resp(e.to_string(), StatusCode::NO_CONTENT);
            } else {
                return build_resp("".to_string(), StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
        Ok(_) => { 
            build_resp( "".to_string() , StatusCode::OK  )
        }
    }
}

