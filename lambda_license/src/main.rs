use lambda_http::service_fn;
use lib_config::config::Config;
use lib_config::logs::setup_log;
use lib_config::traces::setup_tracing_level;
use lib_engage::repositories::sender::SenderEmailsRepo;
use lib_licenses::repositories::owners::OwnerRepo;
use lib_licenses::repositories::shorter::ShorterRepo;
use lib_licenses::services::assets::AssetService;
use lib_licenses::services::owners::OwnerService;
use lib_licenses::services::video::VideoService;
use lib_licenses::repositories::assets::AssetRepo;
use lib_users::repositories::users::UsersRepo;
use lib_users::services::users::UsersService;
use lib_engage::repositories::subscription::SubscriptionRepo;
use lib_engage::services::subscription::SubscriptionService;
use my_lambda::{error::ApiLambdaError, function_handler};

mod my_lambda;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Sync + Send>> {

    setup_log();

    let mut config = Config::new();
    config.setup_with_secrets().await;

    setup_tracing_level(config.env_vars());

    log::info!("bootstrapping dependencies...");

    let asset_repo = AssetRepo::new(&config);
    let shorter_repo = ShorterRepo::new(&config);
    let asset_service = AssetService::new(asset_repo, shorter_repo);

    let owners_repo = OwnerRepo::new(&config);
    let owners_service = OwnerService::new(owners_repo);

    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);

    let video_service = VideoService::new(asset_service.to_owned(), config.to_owned());

    let subscription_repo = SubscriptionRepo::new(&config);
    let sender_repo = SenderEmailsRepo::new(&config);
    let subscription_service = SubscriptionService::new(subscription_repo, sender_repo);
    

    log::info!("bootstrapping dependencies: completed. Lambda ready.");
    let resp = lambda_http::run(service_fn(|event| {
        function_handler(
            &config,
            &asset_service,
            &owners_service,
            //&blockchain_service,
            &user_service,
            &video_service,
            //&tx_service,
            //&license_service,
            //&ledger_service,
            &subscription_service,
            event,
        )
    }))
    .await;

    match resp {
        Ok(r) => Ok(r),
        Err(e) => Err(ApiLambdaError { 0: e.to_string() }.into()),
    }
}
