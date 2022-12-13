// src/main.rs
//use aws_sdk_dynamodb::Client;
use actix_web::{ web, App, HttpServer, };
use http::handlers::{self, AppState};
use actix_web::middleware::Logger;

mod users;
mod config;
mod http;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    let mut config = config::Config::new();
    config.setup().await;

    let user_repo = users::repositories::users::UsersRepo::new(config.getAWSConfig());
    let user_service = users::services::users::UsersService::new(user_repo);

    // Start http server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                user_service: user_service.clone(),
            }))
            .wrap(Logger::default())
            //.wrap(Logger::new("%a %{User-Agent}i"))
            .route("/users", web::get().to(handlers::get_users))
            .route("/users/{id}", web::get().to(handlers::get_user_by_id))
            .route("/users", web::post().to(handlers::add_user))
        //.route("/users/{id}", web::delete().to(handlers::delete_user))
    })
    .bind("127.0.0.1:8080")?
    //.unwrap()
    .run()
    .await

    //let api =  Http;
    //api.start();

    /*
    // Start http server
    HttpServer::new(move || {
        App::new()
            .route("/users", web::get().to(handlers::get_users))
            .route("/users/{id}", web::get().to(handlers::get_user_by_id))
            .route("/users", web::post().to(handlers::add_user))
            //.route("/users/{id}", web::delete().to(handlers::delete_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await

    */
}
