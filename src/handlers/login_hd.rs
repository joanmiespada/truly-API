use actix_web::{web, HttpResponse, Responder};
use aws_sdk_dynamodb::middleware::RetryConfig;
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::{users::{
    errors::users::{DynamoDBError, UserNoExistsError},
    models::user::UserRoles,
}, config};

/*
use crate::users::{
    models::user::{User},
    services::users::{ LoginOps,  UsersService}, errors::users::{DynamoDBError, UserAlreadyExistsError, UserNoExistsError},
};
*/
use super::{appstate::AppState, jwt_middleware::JWTSecurityError};
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

pub async fn login(
    state: web::Data<AppState>,
    payload: web::Json<LoginUser>,
    //path: web::Path<String>,
) -> impl Responder {
    let user_service = &state.user_service;
    let conf = &state.app_config;

    let op_res = user_service
        .login(&payload.device, &payload.email, &payload.password)
        .await;
    match op_res {
        Err(e) => {
            if let Some(_) = e.downcast_ref::<DynamoDBError>() {
                HttpResponse::ServiceUnavailable().finish()
            } else if let Some(e) = e.downcast_ref::<UserNoExistsError>() {
                HttpResponse::NotAcceptable().body(e.to_string())
            } else {
                HttpResponse::InternalServerError().finish()
            }
        }
        Ok(log_inf) => {
            let token_creation_ops = create_jwt(&log_inf.user_id, &log_inf.roles, conf);
            match token_creation_ops {
                Err(e) => HttpResponse::InternalServerError().finish(),
                Ok(token) => HttpResponse::Ok().json(&LoginResponse { token }),
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub uid: String,
    pub roles: Vec<UserRoles>,
    //pub roles: String,
    pub exp: usize,
}

fn create_jwt(uid: &str, roles: &Vec<UserRoles>, conf: &config::Config) -> Result<String, actix_web::Error> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    //let rr = roles.into_iter().map(|r| format!("{},", r.to_string()) ).collect();
    let claims = Claims {
        uid: uid.to_owned(),
        roles: roles.clone(), // rr, // role.to_string(),
        //roles: rr, // role.to_string(),
        exp: expiration as usize,
    };
    let header = Header::new(Algorithm::HS512);
    let key = EncodingKey::from_secret( conf.env_vars().jwt_token_base.as_bytes() );
    let jwt = encode(&header, &claims, &key);
    match jwt {
        Ok(x) => Ok(x),
        Err(e) => Err(JWTSecurityError::from("fail creating a token".to_string()).into() ),
    }
}

pub async fn logout(// state: web::Data<AppState>,
   // payload: web::Json<LoginUser>,
   // path: web::Path<String>,
) -> impl Responder {
    return HttpResponse::Ok().finish();
}


