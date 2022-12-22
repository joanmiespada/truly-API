extern crate crypto;
extern crate rustc_serialize;
extern crate derive_more;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{http::header, App, HttpServer};
use actix_web::{web};
use handlers::appstate::AppState;
use handlers::{login_hd, auth_middleware};
use handlers::users_hd::{self};
use tracing_actix_web::TracingLogger;

mod config;
mod handlers;
mod users;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let mut config = config::Config::new();
    config.setup().await;

    let user_repo = users::repositories::users::UsersRepo::new(&config); 
    let user_service = users::services::users::UsersService::new(user_repo);

    let env = config.env_vars();//.env_variables;//.as_ref().unwrap();
    let server_address = format!("{}:{}", env.local_address, env.local_port );


    // Start http server
    HttpServer::new( move || {
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
    .bind(server_address)//?
    .unwrap_or_else(|_| panic!("Could not bind server to address"))
    //.start();
    .run()
    .await
    
}

fn routes(app: &mut web::ServiceConfig) {
    app
    .service(
        web::scope("/api")
            .wrap(auth_middleware::Auth)
            //.route("/users", web::get().to(users_hd::get_users))
            .route("/users/{id}", web::get().to(users_hd::get_user_by_id))
            .route("/users/{id}", web::put().to(users_hd::update_user))
            .route(
                "/users/promote/{id}",
                web::post().to(users_hd::promote_user), //.and(with_auth(UserRoles::Admin)),
            )
            .route("/users", web::post().to(users_hd::add_user))
            //.route(
            //    "/users/{field}/{value}",
            //    web::get().to(users_hd::get_user_by_filter),
            //),
        //.route("/{id}", web::delete().to( users_hd::delete_user ))
    )
    .service(
        web::scope("/auth")
            .route("/login", web::post().to(login_hd::login))
            //.route("/logout", web::post().to(login_hd::logout)),
    );
}

// -----------------------------------------------------------------------------------------------------
// https://blog.logrocket.com/deploy-lambda-functions-rust/
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use users::models::user::UserRoles;
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
