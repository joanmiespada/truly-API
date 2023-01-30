//extern crate derive_more;
//extern crate rustc_serialize;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::{http::header, App, HttpServer};
use handlers::appstate::AppState;
use handlers::{asset_hd, auth_middleware, jwt_middleware, login_hd, nft_hd, user_my_hd, users_hd};
use lib_config::Config;
use lib_licenses::repositories::assets::AssetRepo;
use lib_licenses::repositories::owners::OwnerRepo;
use lib_licenses::services::assets::AssetService;
use lib_licenses::services::owners::OwnerService;
use log::debug;
use tracing_actix_web::TracingLogger;

use lib_licenses::repositories::ganache::GanacheRepo;
use lib_licenses::services::nfts::NFTsService;

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
            .configure(routes)
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

    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);

    let asset_repo = AssetRepo::new(&config);
    let asset_service = AssetService::new(asset_repo.to_owned());

    let owners_repo = OwnerRepo::new(&config);
    let owners_service = OwnerService::new(owners_repo.to_owned());

    let blockchain = GanacheRepo::new(&config).unwrap();
    let blockchain_service = NFTsService::new(
        blockchain,
        asset_service.to_owned(),
        owners_service.to_owned(),
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

fn routes(app: &mut web::ServiceConfig) {
    app.service(
        web::scope("/api")
            .route(
                "/asset/{id}",
                web::get().to(asset_hd::get_asset_by_token_id),
            )
            .wrap(jwt_middleware::Jwt)
            .route("/asset", web::post().to(asset_hd::create_my_asset))
            .route("/nft/{id}", web::post().to(asset_hd::create_my_nft))
            .route("/asset", web::get().to(asset_hd::get_all_my_assets))
            .route("/user", web::get().to(user_my_hd::get_my_user))
            .route("/user", web::put().to(user_my_hd::update_my_user))
            .route(
                "/user/password",
                web::put().to(user_my_hd::password_update_my_user), //.and(with_auth(UserRoles::Admin)),
            )
            .route("/nft/{id}", web::post().to(nft_hd::add_nft)),
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
