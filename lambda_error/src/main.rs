
use lambda_runtime::{run, service_fn, Error};

use lib_config::{config::Config, traces::setup_tracing_level, 
    logs::setup_log};

use my_lambda::function_handler;

mod my_lambda;

#[tokio::main]
async fn main() -> Result<(), Error> {
    
    setup_log();

    let mut config = Config::new();
    config.setup_with_secrets().await;
    
    setup_tracing_level(config.env_vars());
    
    log::info!("bootstrapping dependencies...");


    run(service_fn(|e| function_handler(
        e, 
        &config, 
    ))).await
}
