pub mod error;
mod get_my_user;
mod update_my_user;
mod update_my_password;

use lambda_http::{
    http::Method, http::StatusCode, IntoResponse, Request, RequestExt,
    Response,
};
use lib_config::Config;
use lib_users::services::users::UsersService;
use lib_util_jwt::{ get_header_jwt };
use serde_json::json;
use tracing::instrument;
use self::get_my_user::get_my_user;
use self::update_my_user::update_my_user;
use self::error::ApiLambdaUserError;
use self::update_my_password::password_update_my_user;

#[instrument]
pub async fn function_handler(
    config: &Config,
    user_service: &UsersService,
    req: Request,
) -> Result<impl IntoResponse, Box<dyn std::error::Error>> {
    let context = req.lambda_context();
    //let query_string = req.query_string_parameters().to_owned();
    //request.uri().path()
    let user_id;
    match check_jwt_token_as_user_logged(&req, config) {
        Err(e) => {
            return build_resp(e.to_string(), StatusCode::FORBIDDEN);
        }
        Ok(id) => user_id = id,
    }

    match req.method() {
        &Method::GET => match req.uri().path() {
            "/api/user" => get_my_user(&req, &context, config, user_service, &user_id).await,
            &_ => build_resp(
                "method not allowed".to_string(),
                StatusCode::METHOD_NOT_ALLOWED,
            ),
        },
        &Method::PUT => match req.uri().path() {
            "/api/user" => update_my_user(&req, &context, config, user_service, &user_id).await,
            "/api/user/password_update" => password_update_my_user(&req, &context, config, user_service, &user_id).await,
            &_ => build_resp(
                "method not allowed".to_string(),
                StatusCode::METHOD_NOT_ALLOWED,
            ),
        },
        _ => build_resp(
            "http verb doesn't use it here".to_string(),
            StatusCode::METHOD_NOT_ALLOWED,
        ),
    }
}
#[tracing::instrument]
fn build_resp(
    msg: String,
    status_code: StatusCode,
) -> Result<Response<String>, Box<dyn std::error::Error>> {
    //} Response<Body> {
    let res = Response::builder()
        .status(status_code)
        .header("content-type", "text/json")
        .body(msg);
    //.map_err(Box::new)?;
    match res {
        Err(e) => Err(ApiLambdaUserError { 0: e.to_string() }.into()),
        Ok(resp) => Ok(resp),
    }
    //Ok(res)
}

fn check_jwt_token_as_user_logged(req: &Request, config: &Config) -> Result<String, String> {
    let user_id;
    let req_headers = req.headers();

    let jwt_secret = config.env_vars().jwt_token_base();
    let claim_ops = get_header_jwt(req_headers, jwt_secret);

    match claim_ops {
        Ok(clm) => {
            user_id = clm.uid;
        }
        Err(e) => {
            return Err(e);
        }
    }
    Ok(user_id)
}
