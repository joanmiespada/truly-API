use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::config::Config;
use lib_config::result::ResultE;
use lib_users::errors::users::{UserDynamoDBError, UserNoExistsError};
use lib_users::services::users::{PromoteUser, UserManipulation, UsersService};

use super::build_resp;
use validator::ValidationError;

//#[instrument]
pub async fn promote_user(
    req: &Request,
    c: &Context,
    config: &Config,
    user_service: &UsersService,
    id: &String,
) -> ResultE<Response<String>> {
    downgrade_upgrade_user(req, c, config, user_service, id, &PromoteUser::Upgrade).await
}

//#[instrument]
pub async fn downgrade_user(
    req: &Request,
    c: &Context,
    config: &Config,
    user_service: &UsersService,
    id: &String,
) -> ResultE<Response<String>> {
    downgrade_upgrade_user(req, c, config, user_service, id, &PromoteUser::Downgrade).await
}

//#[instrument]
async fn downgrade_upgrade_user(
    _req: &Request,
    _c: &Context,
    _config: &Config,
    user_service: &UsersService,
    id: &String,
    grade: &PromoteUser,
) -> ResultE<Response<String>> {
    let op_res = user_service.promote_user_to(id, grade).await;
    match op_res {
        Err(e) => {
            if let Some(e) = e.downcast_ref::<UserDynamoDBError>() {
                return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(e) = e.downcast_ref::<UserNoExistsError>() {
                return build_resp(e.to_string(), StatusCode::NO_CONTENT);
            } else if let Some(m) = e.downcast_ref::<ValidationError>() {
                return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
            } else {
                return build_resp("".to_string(), StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
        Ok(_) => build_resp("".to_string(), StatusCode::OK),
    }
}
