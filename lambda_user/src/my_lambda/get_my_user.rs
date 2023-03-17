use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::config::Config;
use lib_users::errors::users::{UserDynamoDBError, UserNoExistsError};
use lib_users::services::users::{UserManipulation, UsersService};
use serde_json::json;
use tracing::instrument;

use super::build_resp;
use validator::ValidationError;

#[instrument]
pub async fn get_my_user(
    req: &Request,
    _c: &Context,
    config: &Config,
    user_service: &UsersService,
    id: &String,
) -> Result<Response<String>, Box<dyn std::error::Error>> {
    let op_res = user_service.get_by_id(&id).await;
    match op_res {
        Ok(user) => build_resp(json!(user).to_string(), StatusCode::OK),
        Err(e) => {
            if let Some(e) = e.downcast_ref::<UserDynamoDBError>() {
                return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<UserNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else if let Some(m) = e.downcast_ref::<ValidationError>() {
                return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
            } else {
                return build_resp(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }
}
