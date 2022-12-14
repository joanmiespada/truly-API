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

    let user_repo = users::repositories::users::UsersRepo::new(config.aws_config());
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
// -----------------------------------------------------------------------------------------------------
// https://blog.logrocket.com/deploy-lambda-functions-rust/
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // Extract some useful information from the request

    // Return something that implements IntoResponse.
    // It will be serialized to the right response event automatically by the runtime
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body("Hello AWS Lambda HTTP request".into())
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main2() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}