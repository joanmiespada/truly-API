use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::config::Config;
use lib_config::result::ResultE;
use lib_users::errors::users::{UserDynamoDBError, UserNoExistsError};
use lib_users::services::users::{UserManipulation, UsersService};
use serde_json::json;
//use tracing::instrument;

use super::build_resp;

//#[instrument]
pub async fn get_user_by_id(
    _req: &Request,
    _c: &Context,
    _config: &Config,
    user_service: &UsersService,
    id: &String,
) -> ResultE<Response<String>> {
    let op_res = user_service.get_by_id(id).await;
    match op_res {
        Ok(user) => build_resp(json!(user).to_string(), StatusCode::OK),
        Err(e) => {
            if let Some(e) = e.downcast_ref::<UserDynamoDBError>() {
                return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<UserNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
                //HttpResponse::NoContent().finish()
            } else {
                return build_resp(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR);
                //HttpResponse::InternalServerError().finish()
            }
        }
    }
}
