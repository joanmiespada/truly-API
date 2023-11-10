
use lambda_http::Context;
use lambda_http::{http::StatusCode,Request,Response};
use lib_config::config::Config;
use lib_config::environment::{DEV_ENV, STAGE_ENV};
use serde_json::json;

use crate::error::ApiLambdaError;
use crate::jwt::{JWTSecurityError, get_header_jwt};

pub fn jwt_mandatory(req: &Request, config: &Config) -> Result<String, Response<String>> {
    match check_jwt_token_as_user_logged(&req, config) {
        Err(e) => Err(build_resp(e.to_string(), StatusCode::UNAUTHORIZED).unwrap()),
        Ok(id) => Ok(id),
    }
}

pub fn build_resp(
    msg: String,
    status_code: StatusCode,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let res = Response::builder()
        .status(status_code)
        .header("content-type", "text/json")
        .body(msg.clone());
    log::info!("result: {} status code: {}", msg, status_code);
    match res {
        Err(e) => Err(ApiLambdaError { 0: e.to_string() }.into()),
        Ok(resp) => Ok(resp),
    }
}


pub fn build_resp_no_cache(
    msg: String,
    status_code: StatusCode,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let res = Response::builder()
        .status(status_code)
        .header("content-type", "text/json")
        .header("cache-control", "no-cache,max-age=0")
        .body(msg.clone());
    log::info!("result: {} status code: {}", msg, status_code);
    match res {
        Err(e) => Err(ApiLambdaError { 0: e.to_string() }.into()),
        Ok(resp) => Ok(resp),
    }
}

pub fn build_resp_env(
    env: &String,
    error: Box<dyn std::error::Error + Send + Sync>,
    status_code: StatusCode,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let msg: String;
    if env == DEV_ENV || env == STAGE_ENV {
        msg = format!("{}", error);
    } else {
        msg = "".to_string();
    }

    let res = Response::builder()
        .status(status_code)
        .header("content-type", "text/json")
        .header("cache-control", "max-age=300") //5 minutes
        .body(msg);
    match res {
        Err(e) => Err(ApiLambdaError { 0: e.to_string() }.into()),
        Ok(resp) => Ok(resp),
    }
}
pub fn check_jwt_token_as_user_logged(
    req: &Request,
    config: &Config,
) -> Result<String, JWTSecurityError> {
    let user_id;
    let req_headers = req.headers();

    let jwt_secret = config.env_vars().jwt_token_base().unwrap();
    let claim_ops = get_header_jwt(req_headers, &jwt_secret);

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

pub fn not_allowed(
    _req: &Request,
    _c: &Context,
) -> Result<Response<String>, Box<dyn std::error::Error>> {
    let res = Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .header("content-type", "text/json")
        .body(json!({"message":"not allowed."}).to_string());
    //.expect("err creating response");
    //.map_err( |e| ApiLambdaError { 0: e.to_string() }.into()   )?;
    match res {
        Err(e) => Err(ApiLambdaError { 0: e.to_string() }.into()),
        Ok(resp) => Ok(resp),
    }
    //Ok(res);
}
