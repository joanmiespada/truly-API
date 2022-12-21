use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::users::{
    errors::users::{DynamoDBError, JWTSecurityError, UserNoExistsError},
    models::user::UserRoles,
};

/*
use crate::users::{
    models::user::{User},
    services::users::{ LoginOps,  UsersService}, errors::users::{DynamoDBError, UserAlreadyExistsError, UserNoExistsError},
};
*/
use super::appstate::AppState;
use crate::users::services::login::LoginOps;

#[derive(Serialize, Deserialize)]
pub struct LoginUser {
    pub email: Option<String>,
    pub password: Option<String>,
    pub device: Option<String>,
}
#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}
const BEARER: &str = "Bearer ";
const JWT_SECRET: &[u8] = b"secret";

pub async fn login(
    state: web::Data<AppState>,
    payload: web::Json<LoginUser>,
    //path: web::Path<String>,
) -> impl Responder {
    let user_service = &state.user_service;

    let op_res = user_service
        .login(&payload.device, &payload.email, &payload.password)
        .await;
    match op_res {
        Err(e) => {
            if let Some(_) = e.downcast_ref::<DynamoDBError>() {
                HttpResponse::ServiceUnavailable().finish()
            } else if let Some(_) = e.downcast_ref::<UserNoExistsError>() {
                HttpResponse::NotAcceptable().finish()
            } else {
                HttpResponse::InternalServerError().finish()
            }
        }
        Ok(log_inf) => {
            let token_creation_ops = create_jwt(&log_inf.user_id, &log_inf.roles);
            match token_creation_ops {
                Err(e) => HttpResponse::InternalServerError().finish(),
                Ok(token) => HttpResponse::Ok().json(&LoginResponse { token }),
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: String,
    role: String,
    exp: usize,
}

fn create_jwt(uid: &str, roles: &Vec<UserRoles>) -> Result<String, Box<dyn std::error::Error>> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::seconds(60))
        .expect("valid timestamp")
        .timestamp();

    let rr = roles.into_iter().map(|r| format!("{},", r.to_string()) ).collect();
    let claims = Claims {
        sub: uid.to_owned(),
        role: rr, // role.to_string(),
        exp: expiration as usize,
    };
    let header = Header::new(Algorithm::HS512);
    let key = EncodingKey::from_secret(JWT_SECRET);
    let jwt = encode(&header, &claims, &key);
    match jwt {
        Ok(x) => Ok(x),
        Err(e) => Err(JWTSecurityError(e.to_string()).into()),
    }
}

pub async fn logout(// state: web::Data<AppState>,
   // payload: web::Json<LoginUser>,
   // path: web::Path<String>,
) -> impl Responder {
    return HttpResponse::Ok().finish();
}


