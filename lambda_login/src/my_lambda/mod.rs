
use lambda_http::{
     http::Method, http::StatusCode, lambda_runtime::Context,
     service_fn,  IntoResponse, Request, RequestExt, Response,
};
use lib_users::errors::users::{DynamoDBError, UserNoExistsError};
use lib_users::services::login::LoginOps;
use lib_util_jwt::create_jwt;
use serde::{Deserialize, Serialize };
use serde_json::json;
use tracing::{info,trace,debug, instrument};
use lib_config::Config;
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
#[instrument]
pub async fn function_handler(
    config: &Config,
    user_service: &UsersService,
    req: Request,
) -> Result<impl IntoResponse, Box<dyn std::error::Error>> {
    let _context = req.lambda_context();
    //let query_string = req.query_string_parameters().to_owned();
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
#[tracing::instrument]
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

#[tracing::instrument]
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

#[derive(Debug, Deserialize,Serialize, Default)]
pub struct LoginPayload {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub device: Option<String>,
}

#[instrument]
async fn login(
    _req: &Request,
    _c: &Context,
    config: &Config,
    user_service: &UsersService,
) -> Result<Response<String>, Box<dyn std::error::Error>> {
    //let method_name = event.into_parts().0;
    //let args = _req.payload::<LoginPayload>();

    println!("running the login...");
    let payload: LoginPayload = _req.payload()
    .unwrap_or_else(|_parse_err| None)
    .unwrap_or_default();


    // match args{
    //     Err(_) => build_resp("no correct payload attached to the request: either username and password, or device, are mandatories".to_string(), StatusCode::BAD_REQUEST ),
    //     Ok(user_pass) => {
    //         match user_pass {
    //             None => build_resp("either email/password or device fields are empty".to_string(), StatusCode::BAD_REQUEST),
    //             Some(payload) => {
                    println!("payload received for login");
                    let result = user_service.login(&payload.device, &payload.email, &payload.password).await;
                    println!("login service called");
                    match result {
                        Err(e) => {
                            if let Some(_) = e.downcast_ref::<DynamoDBError>() {
                                println!("error: {}",e.to_string());
                                build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE)
                            } else if let Some(e) = e.downcast_ref::<UserNoExistsError>() {
                                let aux2 = serde_json::to_string(&payload).unwrap();
                                let aux1 = format!("error: {} --payload received: {}",e.to_string(), aux2 );
                                println!("{}",aux1);
                                build_resp(aux1, StatusCode::NOT_ACCEPTABLE)
                            } else {
                                println!("error: {}",e.to_string());
                                build_resp(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
                            }
                        }
                        Ok(log_inf) => {
                            
                            let roles: Vec<String> = log_inf.roles.into_iter().map(|ur| ur.to_string()).collect();
                            let token_secret = config.env_vars().jwt_token_base();
                            let token_creation_ops = create_jwt(&log_inf.user_id, &roles, token_secret );
            
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
    //             }
    //         }
    //     }
    // }
}
/* 
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
    
    let resp = lambda_http::run(service_fn(|event | {
        function_handler(config, user_service, event)
    }))
    .await;

    match resp {
        Ok(r) => Ok(r),
        Err(e) => Err(ApiLambdaError { 0: e.to_string() }.into()),
    }

}
*/


