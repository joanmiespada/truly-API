// https://blog.logrocket.com/deploy-lambda-functions-rust/

use lambda_runtime::{run, service_fn, Error};

use lib_config::{config::Config, traces::setup_tracing_level, 
    logs::setup_log};
use lib_engage::{
    repositories::alert_similar::AlertSimilarRepo,
    services::alert_similar::AlertSimilarService
};
use my_lambda::function_handler;

mod my_lambda;

#[tokio::main]
async fn main() -> Result<(), Error> {
    
    setup_log();

    let mut config = Config::new();
    config.setup_with_secrets().await;
    
    setup_tracing_level(config.env_vars());
    
    log::info!("bootstrapping dependencies...");

    let notification_repo = AlertSimilarRepo::new(&config);
    let notification_service = AlertSimilarService::new(notification_repo);

    run(service_fn(|e| function_handler(e, &config, &notification_service))).await
}
