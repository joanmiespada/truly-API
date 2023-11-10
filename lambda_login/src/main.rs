
use lambda_http::service_fn;
use lib_config::{config::Config, traces::setup_tracing_level, 
    logs::setup_log};
use lib_users::repositories::users::UsersRepo;
use lib_users::services::users::UsersService;
use lib_util_jwt::error::ApiLambdaError;
use my_lambda::function_handler;

mod my_lambda;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    setup_log();

    let mut config = Config::new();
    config.setup_with_secrets().await;

    setup_tracing_level(config.env_vars());

    log::info!("bootstrapping dependencies...");

    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);

    log::info!("lambda ready, awaiting for events.");
    let resp = lambda_http::run(service_fn(|event| {
        function_handler(&config, &user_service, event)
    }))
    .await;

    match resp {
        Ok(r) => Ok(r),
        Err(e) => Err(ApiLambdaError { 0: e.to_string() }.into()),
    }
}
