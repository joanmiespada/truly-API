mod login;
mod signup;

use self::signup::create_basic_user;
use lambda_http::{
    http::Method, http::StatusCode, IntoResponse, Request, RequestExt,
    Response,
};
use lib_config::{config::Config, stage::remove_stage_prefix};
use lib_users::services::users::UsersService;
use lib_util_jwt::{error::ApiLambdaError, build::not_allowed};
use login::login;

//#[instrument]
pub async fn function_handler(
    config: &Config,
    user_service: &UsersService,
    req: Request,
) -> Result<impl IntoResponse, Box<dyn std::error::Error>> {
    let context = req.lambda_context();

    let path = remove_stage_prefix( 
        req.uri().path().to_string(), 
        config.env_vars().api_stage().clone().unwrap());

    match req.method() {
        &Method::POST => match path.as_str()  {
            "/auth/login" => login(&req, &context, config, user_service).await,
            "/auth/signup" => create_basic_user(&req, &context, config, user_service).await,
            _ => not_allowed(&req, &context),
        },
        _ => not_allowed(&req, &context),
    }
}
//#[tracing::instrument]
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

//#[tracing::instrument]

