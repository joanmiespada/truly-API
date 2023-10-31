
use lambda_notifications::function_handler;
use lambda_runtime::{run, service_fn, Error};

use lib_config::{config::Config, traces::setup_tracing_level, 
    logs::setup_log};
use lib_engage::{
    repositories::{alert_similar::AlertSimilarRepo, subscription::SubscriptionRepo, sender::SenderEmailsRepo},
    services::{alert_similar::AlertSimilarService, subscription::SubscriptionService}
};
use lib_licenses::{repositories::{assets::AssetRepo, shorter::ShorterRepo}, services::assets::AssetService};
use lib_users::{repositories::users::UsersRepo, services::users::UsersService};


#[tokio::main]
async fn main() -> Result<(), Error> {
    
    setup_log();

    let mut config = Config::new();
    config.setup_with_secrets().await;
    
    setup_tracing_level(config.env_vars());
    
    log::info!("bootstrapping dependencies...");

    let alert_repo = AlertSimilarRepo::new(&config);
    let alert_service = AlertSimilarService::new(alert_repo);

    let subscription_repo = SubscriptionRepo::new(&config);
    let sender_emails_repo = SenderEmailsRepo::new(&config);
    let subscription_service = SubscriptionService::new(subscription_repo, sender_emails_repo );

    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);

    let asset_repo = AssetRepo::new(&config);
    let shorter_repo = ShorterRepo::new(&config);
    let asset_service = AssetService::new(asset_repo, shorter_repo);

    let aux =SenderEmailsRepo::new(&config);

    run(service_fn(|e| function_handler(
        e, 
        &alert_service, 
        &subscription_service,
        &user_service,
        &asset_service,
        &aux
    ))).await
}
