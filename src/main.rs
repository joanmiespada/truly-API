// src/main.rs
//use aws_sdk_dynamodb::Client;
//use http::server::{Http, Starter};
use actix_web::{dev::Server, web, App, HttpServer, Error};
use http::handlers::{self, AppState};

mod users;
mod config;
mod http;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    config::configuration();

    let userRepo = users::repositories::users::UsersRepo::new();
    let userService = users::services::users::UsersService::new(userRepo);

    // Start http server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                user_service: userService.clone(),
            }))
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
