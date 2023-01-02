extern crate derive_more;
extern crate rustc_serialize;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::{http::header, App, HttpServer};
use config::{ ENV_VAR_MODE_HTTP_SERVER, ENV_VAR_MODE_LAMBDA};
use handlers::appstate::AppState;
use handlers::{auth_middleware, jwt_middleware, login_hd, user_my_hd, users_hd};
use tracing_actix_web::TracingLogger;

use users::services::users::UsersService;

mod config;
mod handlers;
mod users;
mod lambda;

async fn http_server(config: config::Config, user_service: UsersService) {
    let env = config.env_vars();
    let server_address = format!("{}:{}", env.local_address, env.local_port);

    // Start http server
    let _ = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                user_service: user_service.clone(),
                app_config: config.clone(),
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

//#[actix_rt::main]
#[tokio::main]
async fn main() -> Result<(),Box<dyn std::error::Error>> {
     
    let mut config = config::Config::new();
    config.setup().await;

    let user_repo = users::repositories::users::UsersRepo::new(&config);
    let user_service = users::services::users::UsersService::new(user_repo);

    /*if config.env_vars().mode ==  ENV_VAR_MODE_HTTP_SERVER {
        http_server(config, user_service).await
    } else if config.env_vars().mode == ENV_VAR_MODE_LAMBDA {
        lambda::lambda_main(&config, &user_service).await
    } else{
        panic!("no mode set up at env vars")
    }*/

    lambda::lambda_main(&config, &user_service).await
    

}

fn routes(app: &mut web::ServiceConfig) {
    app.service(
        web::scope("/api")
            .wrap(jwt_middleware::Jwt)
            .route("/user", web::get().to(user_my_hd::get_my_user))
            .route("/user", web::put().to(user_my_hd::update_my_user))
            .route(
                "/user/password_update",
                web::put().to(user_my_hd::password_update_my_user), //.and(with_auth(UserRoles::Admin)),
            ),
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

