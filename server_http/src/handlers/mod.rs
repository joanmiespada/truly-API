
use actix_web::{http::{self, StatusCode}, HttpResponse, Responder};

pub mod users_hd;
pub mod user_my_hd;
pub mod login_hd;
pub mod appstate;
pub mod auth_middleware;
pub mod jwt_middleware;
pub mod asset_hd;
pub mod nft_hd;

pub fn build_resp(body: String, code: http::StatusCode) -> impl Responder {

    match code {
        StatusCode::BAD_REQUEST => HttpResponse::BadRequest().body(body),
        StatusCode::OK => HttpResponse::Ok().body(body), 
        StatusCode::SERVICE_UNAVAILABLE => HttpResponse::ServiceUnavailable().body(body),
        StatusCode::NO_CONTENT => HttpResponse::NoContent().body(body),
        StatusCode::INTERNAL_SERVER_ERROR => HttpResponse::InternalServerError().body(body),
        _ => HttpResponse::InternalServerError().body("no status code has been implemented!"),
    }
}