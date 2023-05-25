use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::{http::header, App, HttpServer};
use handlers::appstate::AppState;
use handlers::{asset_hd, auth_middleware, jwt_middleware, login_hd, nft_hd, user_my_hd, users_hd};
use lib_blockchain::repositories::block_tx::BlockchainTxRepo;
use lib_blockchain::repositories::blockchain::BlockchainRepo;
use lib_blockchain::repositories::contract::ContractRepo;
use lib_blockchain::repositories::keypairs::KeyPairRepo;
use lib_blockchain::services::block_tx::BlockchainTxService;
use lib_config::config::Config;
use lib_licenses::repositories::assets::AssetRepo;
use lib_licenses::repositories::owners::OwnerRepo;
use lib_licenses::repositories::shorter::ShorterRepo;
use lib_licenses::services::assets::AssetService;
use lib_licenses::services::owners::OwnerService;
use log::debug;
use tracing_actix_web::TracingLogger;

use lib_blockchain::blockchains::ganache::GanacheRepo;
use lib_blockchain::services::nfts::NFTsService;

use lib_users::repositories::users::UsersRepo;
use lib_users::services::users::UsersService;

mod handlers;

const DEFAULT_ADDRESS: &str = "0.0.0.0";
const DEFAULT_PORT: &str = "8080";

async fn http_server(
    config: Config,
    user_service: UsersService,
    asset_service: AssetService,
    owner_service: OwnerService,
    blockchain_service: NFTsService,
) {
    let server_address = format!("{}:{}", DEFAULT_ADDRESS, DEFAULT_PORT);

    debug!("Server up and running {}", server_address);

    // Start http server
    let _ = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                user_service: user_service.clone(),
                app_config: config.clone(),
                asset_service: asset_service.clone(),
                owner_service: owner_service.clone(),
                blockchain_service: blockchain_service.clone(),
            }))
            .wrap(
                Cors::default()
                    //.allowed_origin("http://localhost:8080")
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST", "PUT"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
            //.wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(TracingLogger::default())
            .wrap(Logger::default())
            .configure(|srv_conf| routes(srv_conf, &config))
    })
    .bind(server_address) //?
    .unwrap_or_else(|_| panic!("Could not bind server to address"))
    //.start();
    .run()
    .await;
}

#[actix_rt::main]
async fn main() {
    //-> Result<(),Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let mut config = Config::new();
    config.setup_with_secrets().await;

    let repo_tx = BlockchainTxRepo::new(&config.to_owned());
    let tx_service = BlockchainTxService::new(repo_tx);

    let user_repo = UsersRepo::new(&config.to_owned());
    let user_service = UsersService::new(user_repo);

    let asset_repo = AssetRepo::new(&config);
    let shorter_repo = ShorterRepo::new(&config);
    let asset_service = AssetService::new(asset_repo.to_owned(), shorter_repo.to_owned());

    let owners_repo = OwnerRepo::new(&config);
    let owners_service = OwnerService::new(owners_repo.to_owned());

    let key_repo = KeyPairRepo::new(&config);

    let blockchains_repo = BlockchainRepo::new(&config);
    let contracts_repo = ContractRepo::new(&config);

    let blockchain = GanacheRepo::new(&config, &contracts_repo, &blockchains_repo)
        .await
        .unwrap();

    let blockchain_service = NFTsService::new(
        blockchain,
        key_repo.to_owned(),
        asset_service.to_owned(),
        owners_service.to_owned(),
        tx_service.to_owned(),
        config.to_owned(),
    );

    http_server(
        config,
        user_service,
        asset_service,
        owners_service,
        blockchain_service,
    )
    .await;
}

fn routes(app: &mut web::ServiceConfig, _config: &Config) {
    app.service(
        web::scope("/api")
            .route(
                "/asset/{id}",
                web::get().to(asset_hd::get_asset_by_token_id),
            )
            .wrap(jwt_middleware::Jwt)
            .route("/asset", web::post().to(asset_hd::create_my_asset))
            .route("/asset", web::get().to(asset_hd::get_all_my_assets))
            .route("/user", web::get().to(user_my_hd::get_my_user))
            .route("/user", web::put().to(user_my_hd::update_my_user))
            .route(
                "/user/password",
                web::put().to(user_my_hd::password_update_my_user),
            )
            .route("/nft", web::post().to(nft_hd::add_nft)),
    )
    .service(
        web::scope("/admin")
            .wrap(auth_middleware::Auth)
            .route("/users", web::get().to(users_hd::get_users))
            .route("/users/{id}", web::get().to(users_hd::get_user_by_id))
            .route("/users/{id}", web::put().to(users_hd::update_user))
            .route(
                "/users/upgrade/{id}",
                web::post().to(users_hd::promote_user), //.and(with_auth(UserRoles::Admin)),
            )
            .route(
                "/users/downgrade/{id}",
                web::post().to(users_hd::downgrade_user), //.and(with_auth(UserRoles::Admin)),
            )
            .route(
                "/users/password_update/{id}",
                web::post().to(users_hd::password_update_user), //.and(with_auth(UserRoles::Admin)),
            ),
    )
    .service(
        web::scope("/auth")
            .route("/login", web::post().to(login_hd::login))
            .route("/signup", web::post().to(users_hd::add_user)), //.route("/logout", web::post().to(login_hd::logout)),
    );
}
