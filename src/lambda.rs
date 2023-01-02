// -----------------------------------------------------------------------------------------------------
// https://blog.logrocket.com/deploy-lambda-functions-rust/
use http::StatusCode;
use lambda_http::{
    aws_lambda_events, http::HeaderMap, http::Method, lambda_runtime::Context, run, service_fn,
    tower::ServiceBuilder, Body, Error, IntoResponse, Request, RequestExt, Response,
};
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};
//use aws_lambda_events::encodings::Body;
//use http::header::HeaderMap;

use crate::{config, users::services::users::UsersService};

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
    config: &config::Config,
    user_service: &UsersService,
    req: Request,
) -> Result<impl IntoResponse, Error> {
    let _context = req.lambda_context();
    let query_string = req.query_string_parameters().to_owned();
     not_allowed(&req, &_context)
     /* 
    match req.method().as_str() {
        "POST" => login(&req, &_context, config, user_service),
        "GET" =>  not_allowed(&req, &_context),
        _ => Ok(build_resp("".to_owned(), StatusCode::METHOD_NOT_ALLOWED))
    }*/
}

fn build_resp(msg: String, status_code: StatusCode) -> Response<Body> {
    Response::builder()
        .status(status_code)
        .body(msg.into())
        .expect("err creating response")
}

fn not_allowed(_req: &Request, _c: &Context) -> Result<impl IntoResponse, Error> {
    let res = Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .header("content-type", "text/json")
        .body(json!({"a":"0"}).to_string())
        //.expect("err creating response");
        .map_err(Box::new)?;
    Ok(res)
}

fn login(
    _req: &Request,
    _c: &Context,
    config: &config::Config,
    user_service: &UsersService,
) -> Result<impl IntoResponse, Error> {
    //let method_name = event.into_parts().0;
    //let body = event.payload::<MyPayload>()?;
    // Extract some useful information from the request

    // Return something that implements IntoResponse.
    // It will be serialized to the right response event automatically by the runtime
    let resp = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/json")
        .body(json!({"message":"Hello AWS 123 Lambda HTTP request"}).to_string())
        .map_err(Box::new)?;

    Ok(resp)
}

//#[actix_rt::main]
//#[tokio::main]
pub async fn lambda_main(
    config: &config::Config,
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

    let cors_layer = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_origin(Any);

    let handler = ServiceBuilder::new()
        // Add the CORS layer to the service
        .layer(cors_layer)
        .service(service_fn(|event: Request| {
            function_handler(config, user_service, event)
        }));

    //let _ = lambda_http::run(handler).await;
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
