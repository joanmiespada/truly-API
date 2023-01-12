use std::str::FromStr;

use lambda_http::Context;
use lambda_http::{http::Method, http::StatusCode, IntoResponse, Request, RequestExt, Response};
use lib_config::Config;
use lib_users::models::user::UserRoles;
use lib_users::services::users::UsersService;
use lib_util_jwt::get_header_jwt;
use tracing::{info, instrument, trace};

use self::downgrade_user::downgrade_user;
use self::error::ApiLambdaAdminUserError;
use self::get_user_by_id::get_user_by_id;
use self::get_users::get_users;
use self::password_update_user::password_update_user;
use self::promote_user::promote_user;
use self::update_user::update_user;
use matchit::Router;

mod downgrade_user;
pub mod error;
mod get_user_by_id;
mod get_users;
mod password_update_user;
mod promote_user;
mod update_user;

#[instrument]
pub async fn function_handler(
    config: &Config,
    user_service: &UsersService,
    req: Request,
) -> Result<impl IntoResponse, Box<dyn std::error::Error>> {
    let context = req.lambda_context();
    //let query_string = req.query_string_parameters().to_owned();
    //request.uri().path()

    match check_jwt_token_as_admin(&req, config) {
        Err(e) => {
            return build_resp(e.to_string(), StatusCode::FORBIDDEN);
        }
        Ok(_) => {}
    }

    let mut router = Router::new();
    router.insert("/admin/users", Some("1"))?;
    router.insert("/admin/users/:id", Some("2"))?;

    //info!("{}",req.uri().path());
    match req.method() {
        &Method::GET => match router.at(req.uri().path()) {
            Err(e) => build_resp(
                "method not allowed".to_string(),
                StatusCode::METHOD_NOT_ALLOWED,
            ),
            Ok(matched) => {
                match matched.value.unwrap() {
                    "1" => get_users(&req, &context, config, user_service).await,
                    "2" => { 
                        let aux = matched.params.get("id").unwrap().to_string();
                        return get_user_by_id(&req, &context, config, user_service, &aux).await;
                    },
                    _ => build_resp(
                        "method not allowed".to_string(),
                        StatusCode::METHOD_NOT_ALLOWED,
                    ),
                }
                
            }
        },
        &Method::POST => match req.uri().path() {
            "/users/password_update/{id}" => {
                password_update_user(&req, &context, config, user_service).await
            }
            "/users/upgrade/{id}" => promote_user(&req, &context, config, user_service).await,
            "/users/downgrade/{id}" => downgrade_user(&req, &context, config, user_service).await,
            &_ => build_resp(
                "method not allowed".to_string(),
                StatusCode::METHOD_NOT_ALLOWED,
            ),
        },
        &Method::PUT => match req.uri().path() {
            "/admin/users/{id}" => update_user(&req, &context, config, user_service).await,
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
        Err(e) => Err(ApiLambdaAdminUserError { 0: e.to_string() }.into()),
        Ok(resp) => Ok(resp),
    }
    //Ok(res)
}

fn check_jwt_token_as_admin(req: &Request, config: &Config) -> Result<bool, String> {
    let auth_flag;
    let req_headers = req.headers();

    let jwt_secret = config.env_vars().jwt_token_base();
    let claim_ops = get_header_jwt(req_headers, jwt_secret);

    match claim_ops {
        Ok(clm) => {
            let matches = clm
                .roles
                .into_iter()
                .map(|i| UserRoles::deserialize(i.as_str()).unwrap())
                .filter(|i| i.is_admin())
                .count();
            if matches == 0 {
                auth_flag = false;
            } else {
                auth_flag = true;
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
    Ok(auth_flag)
}
