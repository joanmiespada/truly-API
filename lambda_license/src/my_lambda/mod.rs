pub mod error;
mod assets;
mod owners;

use std::str::FromStr;

use lambda_http::{
    http::Method, http::StatusCode, IntoResponse, Request, RequestExt,
    Response,
};
use lib_config::Config;
use lib_util_jwt::{get_header_jwt, JWTSecurityError};
use tracing::instrument;
use assets::get_my_asset::{get_my_asset, get_my_assets_all};
//use self::assets::update_my_asset::update_my_asset;
use self::assets::create_my_asset::create_my_asset;
use self::error::ApiLambdaError;
use lib_licenses::services::assets::AssetService;
use lib_licenses::services::owners::OwnerService;
use matchit::Router;
use uuid::Uuid;

#[instrument]
pub async fn function_handler(
    config: &Config,
    asset_service: &AssetService,
    owners_service: &OwnerService,
    req: Request,
) -> Result<impl IntoResponse, Box<dyn std::error::Error>> {
    let context = req.lambda_context();
    //let query_string = req.query_string_parameters().to_owned();
    //request.uri().path()
    let user_id;
    match check_jwt_token_as_user_logged(&req, config) {
        Err(e) => {
            return build_resp(e.to_string(), StatusCode::UNAUTHORIZED);
        }
        Ok(id) => user_id = id,
    }

    let mut router = Router::new();
    router.insert("/api/asset", Some("1"))?;
    router.insert("/api/asset/:id", Some("2"))?;



    match req.method() {
        &Method::GET => match router.at(req.uri().path()) {
            Err(_) => build_resp(
                "method not allowed".to_string(),
                StatusCode::METHOD_NOT_ALLOWED,
            ),
            Ok(matched) => match matched.value.unwrap() {
                "1" => get_my_assets_all(&req, &context, config, asset_service,owners_service, &user_id).await ,
                "2" => {
                    let id = matched.params.get("id").unwrap().to_string();
                    let asset_id = Uuid::from_str(id.as_str())?;
                    return get_my_asset(&req, &context, config, asset_service,owners_service, &asset_id, &user_id).await;

                }
                _ => build_resp(
                    "method not allowed".to_string(),
                    StatusCode::METHOD_NOT_ALLOWED,
                ),
            },
        },
        &Method::POST => match req.uri().path() {
            "1" => create_my_asset(&req, &context, config, asset_service,owners_service, &user_id).await,
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
        Err(e) => Err(ApiLambdaError { 0: e.to_string() }.into()),
        Ok(resp) => Ok(resp),
    }
    //Ok(res)
}

fn check_jwt_token_as_user_logged(req: &Request, config: &Config) -> Result<String, JWTSecurityError > {
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
