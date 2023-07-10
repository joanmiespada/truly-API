mod login;
mod signup;

use self::signup::create_basic_user;
use lambda_http::{
    http::Method, http::StatusCode, lambda_runtime::Context, IntoResponse, Request, RequestExt,
    Response,
};
use lib_config::config::Config;
use lib_users::services::users::UsersService;
use login::login;
use serde_json::json;
use tracing::instrument;
use log::{debug,info};

#[derive(Debug)]
pub struct ApiLambdaError(pub String);

impl std::error::Error for ApiLambdaError {}

impl std::fmt::Display for ApiLambdaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "api lambda error: {}", self.0)
    }
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
#[instrument]
pub async fn function_handler(
    config: &Config,
    user_service: &UsersService,
    req: Request,
) -> Result<impl IntoResponse, Box<dyn std::error::Error>> {
    let context = req.lambda_context();
    //let query_string = req.query_string_parameters().to_owned();
    //request.uri().path()
    debug!("debug - uri {}", req.uri().path());
    info!("info - uri {}", req.uri().path());
    info!("{:#?}", req);

    let path = remove_api_prefix( 
        req.uri().path().to_string(), 
        config.env_vars().api_stage().unwrap() );

    info!("info - path {}", path);

    match req.method() {
        &Method::POST => match path.as_str()  {
            "/auth/login" => login(&req, &context, config, user_service).await,
            "/auth/signup" => create_basic_user(&req, &context, config, user_service).await,
            _ => not_allowed(&req, &context),
        },
        _ => not_allowed(&req, &context),
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

#[tracing::instrument]
fn not_allowed(
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

fn remove_api_prefix(input:String, pattern: String) -> String {

    let last_v1_index = input.rfind(pattern);
    let result = match last_v1_index {
        Some(index) => input[(index + pattern.len())..].to_string(),
        None => input.to_string(),
    };

    result

}

#[tokio::test]
async fn test_remove_api_prefix() {

    let pattern = "/v1".to_string();

    let value= "/v1/v1/abc/cvf".to_string();
    let aux = remove_api_prefix(value, pattern);
    assert_eq!(aux,"/abc/cvf");

    let value= "/abc/cvf".to_string();
    let aux = remove_api_prefix(value, pattern);
    assert_eq!(aux,"/abc/cvf");

}