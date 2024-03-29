use lambda_http::{http::Method, http::StatusCode, IntoResponse, Request, RequestExt};
use lib_config::config::Config;
use lib_config::result::ResultE;
use lib_users::services::users::UsersService;
use lib_util_jwt::build::build_resp;
use lib_util_jwt::jwt::check_jwt_token_as_admin;
use self::get_user_by_id::get_user_by_id;
use self::get_users::get_users;
use self::password_update_user::password_update_user;
use self::promote_user::{downgrade_user, promote_user};
use self::update_user::update_user;
use matchit::Router;

pub mod error;
mod get_user_by_id;
mod get_users;
mod password_update_user;
mod promote_user;
mod update_user;

//#[instrument]
pub async fn function_handler(
    config: &Config,
    user_service: &UsersService,
    req: Request,
) -> ResultE<impl IntoResponse> {
    let context = req.lambda_context();
    //let query_string = req.query_string_parameters().to_owned();
    //request.uri().path()

    match check_jwt_token_as_admin(&req, config) {
        Err(e) => {
            return build_resp(e.to_string(), StatusCode::UNAUTHORIZED);
        }
        Ok(value) => match value {
            false => {
                return build_resp(
                    "you aren't admin, please login as admin".to_string(),
                    StatusCode::UNAUTHORIZED,
                );
            }
            _ => {}
        },
    }

    let mut router = Router::new();
    router.insert("/admin/users", Some("1"))?;
    router.insert("/admin/users/:id", Some("2"))?;
    router.insert("/admin/users/password_update/:id", Some("3"))?;
    router.insert("/admin/users/upgrade/:id", Some("4"))?;
    router.insert("/admin/users/downgrade/:id", Some("5"))?;

    //info!("{}",req.uri().path());
    match req.method() {
        &Method::GET => match router.at(req.uri().path()) {
            Err(_) => build_resp(
                "method not allowed".to_string(),
                StatusCode::METHOD_NOT_ALLOWED,
            ),
            Ok(matched) => match matched.value.unwrap() {
                "1" => get_users(&req, &context, config, user_service).await,
                "2" => {
                    let id = matched.params.get("id").unwrap().to_string();
                    return get_user_by_id(&req, &context, config, user_service, &id).await;
                }
                _ => build_resp(
                    "method not allowed".to_string(),
                    StatusCode::METHOD_NOT_ALLOWED,
                ),
            },
        },
        &Method::POST => match router.at(req.uri().path()) {
            Err(_) => build_resp(
                "method not allowed".to_string(),
                StatusCode::METHOD_NOT_ALLOWED,
            ),
            Ok(matched) => match matched.value.unwrap() {
                "3" => {
                    let id = matched.params.get("id").unwrap().to_string();
                    return password_update_user(&req, &context, config, user_service, &id).await;
                }
                "4" => {
                    let id = matched.params.get("id").unwrap().to_string();
                    return promote_user(&req, &context, config, user_service, &id).await;
                }
                "5" => {
                    let id = matched.params.get("id").unwrap().to_string();
                    return downgrade_user(&req, &context, config, user_service, &id).await;
                }
                _ => build_resp(
                    "method not allowed".to_string(),
                    StatusCode::METHOD_NOT_ALLOWED,
                ),
            },
        },
        &Method::PUT => match router.at(req.uri().path()) {
            Err(_) => build_resp(
                "method not allowed".to_string(),
                StatusCode::METHOD_NOT_ALLOWED,
            ),
            Ok(matched) => match matched.value.unwrap() {
                "2" => {
                    let id = matched.params.get("id").unwrap().to_string();
                    update_user(&req, &context, config, user_service, &id).await
                }
                &_ => build_resp(
                    "method not allowed".to_string(),
                    StatusCode::METHOD_NOT_ALLOWED,
                ),
            },
        },
        _ => build_resp(
            "http verb doesn't use it here".to_string(),
            StatusCode::METHOD_NOT_ALLOWED,
        ),
    }
}


