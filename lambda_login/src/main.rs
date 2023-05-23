// https://blog.logrocket.com/deploy-lambda-functions-rust/

use lambda_http::service_fn;
use lib_config::config::Config;
use lib_users::repositories::users::UsersRepo;
use lib_users::services::users::UsersService;
use my_lambda::{function_handler, ApiLambdaError};

mod my_lambda;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let mut config = Config::new();
    config.setup_with_secrets().await;

    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);

    // my_lambda::lambda_main(&config, &user_service).await

    let resp = lambda_http::run(service_fn(|event| {
        function_handler(&config, &user_service, event)
    }))
    .await;

    match resp {
        Ok(r) => Ok(r),
        Err(e) => Err(ApiLambdaError { 0: e.to_string() }.into()),
    }
}
