
use actix_web::{http::{self, StatusCode}, HttpResponse, Responder};
use serde::Serialize;

pub mod users_hd;
pub mod user_my_hd;
pub mod login_hd;
pub mod appstate;
pub mod auth_middleware;
pub mod jwt_middleware;
pub mod asset_hd;
pub mod nft_hd;

pub fn build_resp(body: impl Serialize, code: http::StatusCode) -> impl Responder {

    match code {
        StatusCode::BAD_REQUEST => HttpResponse::BadRequest().json(body),
        StatusCode::OK => HttpResponse::Ok().json(body), 
        StatusCode::SERVICE_UNAVAILABLE => HttpResponse::ServiceUnavailable().json(body),
        StatusCode::NO_CONTENT => HttpResponse::NoContent().json(body),
        StatusCode::INTERNAL_SERVER_ERROR => HttpResponse::InternalServerError().json(body),
        StatusCode::NOT_ACCEPTABLE => HttpResponse::NotAcceptable().json(body),
        _ => HttpResponse::InternalServerError().body("no status code has been implemented!"),
    }
}

