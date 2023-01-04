
// https://blog.logrocket.com/deploy-lambda-functions-rust/
use lambda_http::{
     http::Method, http::StatusCode, lambda_runtime::Context,
    run, service_fn,  IntoResponse, Request, RequestExt, Response,
};
use lib_users::errors::users::{DynamoDBError, UserNoExistsError};
use lib_users::services::login::LoginOps;
use lib_util_jwt::create_jwt;
//use lib_util_jwt::create_jwt;
use serde::Deserialize;
use serde_json::json;
use std::error::Error;
//use serde_json::json;
//use tower_http::cors::{Any, CorsLayer};
//use aws_lambda_events::encodings::Body;
//use http::header::HeaderMap;

use lib_config::Config;
use lib_users::repositories::users::UsersRepo;
use lib_users::services::users::UsersService;

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
async fn function_handler(
    config: &Config,
    user_service: &UsersService,
    req: Request,
) -> Result<impl IntoResponse, Box<dyn std::error::Error>> {
    let _context = req.lambda_context();
    let query_string = req.query_string_parameters().to_owned();
    //not_allowed(&req, &_context)
    //request.uri().path()
    match req.method() {
        &Method::POST => login(&req, &_context, config, user_service).await,
        &Method::GET => not_allowed(&req, &_context),
        _ => build_resp(
            "http verb doesn't use it here".to_string(),
            StatusCode::METHOD_NOT_ALLOWED,
        ),
    }
}

fn build_resp(
    msg: String,
    status_code: StatusCode,
) -> Result<Response<String>, Box<dyn std::error::Error>> {
    //} Response<Body> {
    let res = Response::builder()
        .status(status_code)
        .header("content-type", "text/json")
        .body(json!({ "message": msg }).to_string());
    //.map_err(Box::new)?;
    match res {
        Err(e) => Err(ApiLambdaError { 0: e.to_string() }.into()),
        Ok(resp) => Ok(resp),
    }
    //Ok(res)
}

fn not_allowed(
    _req: &Request,
    _c: &Context,
) -> Result<Response<String>, Box<dyn std::error::Error>> {
    let res = Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .header("content-type", "text/json")
        .body(json!({"message":"not allowed"}).to_string());
    //.expect("err creating response");
    //.map_err( |e| ApiLambdaError { 0: e.to_string() }.into()   )?;
    match res {
        Err(e) => Err(ApiLambdaError { 0: e.to_string() }.into()),
        Ok(resp) => Ok(resp),
    }
    //Ok(res);
}

#[derive(Deserialize)]
pub struct LoginPayload {
    pub email: Option<String>,
    pub password: Option<String>,
    pub device: Option<String>,
}

async fn login(
    _req: &Request,
    _c: &Context,
    config: &Config,
    user_service: &UsersService,
) -> Result<Response<String>, Box<dyn std::error::Error>> {
    //let method_name = event.into_parts().0;
    let args = _req.payload::<LoginPayload>();
    match args{
        Err(e) => build_resp("no correct payload attached to the request: either username and password, or device, are mandatories".to_string(), StatusCode::BAD_REQUEST ),
        Ok(user_pass) => {
            match user_pass {
                None => build_resp("username or password fields are empty".to_string(), StatusCode::BAD_REQUEST),
                Some(payload) => {
                    let result = user_service.login(&payload.device, &payload.email, &payload.password).await;
                    match result {
                        Err(e) => {
                            if let Some(_) = e.downcast_ref::<DynamoDBError>() {
                                build_resp("".to_string(), StatusCode::SERVICE_UNAVAILABLE)
                            } else if let Some(e) = e.downcast_ref::<UserNoExistsError>() {
                                build_resp(e.to_string(), StatusCode::NOT_ACCEPTABLE)
                            } else {
                                build_resp("".to_string(), StatusCode::INTERNAL_SERVER_ERROR)
                            }
                        }
                        Ok(log_inf) => {
                            
                            let roles: Vec<String> = log_inf.roles.into_iter().map(|ur| ur.to_string()).collect();
                            let token_secret = &config.env_vars().jwt_token_base;
                            let token_creation_ops = create_jwt(&log_inf.user_id, &roles, token_secret);
            
                            match token_creation_ops {
                                Err(_) => build_resp("".to_string(), StatusCode::INTERNAL_SERVER_ERROR),
                                Ok(token) => {
                                    let final_answer = Response::builder()
                                        .status(StatusCode::OK)
                                        .header("content-type", "text/json")
                                        .body(json!({ "token": token }).to_string());
                                    match final_answer {
                                        Err(e) => Err(ApiLambdaError { 0: e.to_string() }.into()),
                                        Ok(resp) => Ok(resp),
                                    }
                                }
                            }
                            
                        }
                    }
                }
            }
        }
    }
}

pub async fn lambda_main(
    config: &Config,
    user_service: &UsersService,
) -> Result<(), Box<dyn std::error::Error>> {
    // -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    /*let cors_layer = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT])
        .allow_origin(Any);


    let handler = ServiceBuilder::new()
        // Add the CORS layer to the service
        .layer(cors_layer)
        .service(service_fn(|event: Request| {
            function_handler(config, user_service, event)
        }));*/

    //let resp = lambda_http::run(handler).await;
    let resp = lambda_http::run(service_fn(|event: Request| {
        function_handler(config, user_service, event)
    }))
    .await;

    match resp {
        Ok(r) => Ok(r),
        Err(e) => Err(ApiLambdaError { 0: e.to_string() }.into()),
    }

    // return resp;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::new();
    config.setup().await;

    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);

    lambda_main(&config, &user_service).await
}
