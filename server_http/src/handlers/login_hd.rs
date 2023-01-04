use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use lib_config::{Config};
use lib_users::{
        errors::users::{DynamoDBError, UserNoExistsError},
        models::user::UserRoles,
};

use super::{appstate::AppState, jwt_middleware::JWTSecurityError};
use lib_users::services::login::LoginOps;

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

pub async fn login(state: web::Data<AppState>, payload: web::Json<LoginUser>) -> impl Responder {
    let user_service = &state.user_service;
    let conf: &lib_config::Config = &state.app_config;

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
                Err(_) => HttpResponse::InternalServerError().finish(),
                Ok(token) => HttpResponse::Ok().json(&LoginResponse { token }),
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub uid: String,
    pub roles: Vec<UserRoles>,
    pub exp: usize,
}

fn create_jwt(
    uid: &str,
    roles: &Vec<UserRoles>,
    conf: &Config,
) -> Result<String, actix_web::Error> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        uid: uid.to_owned(),
        roles: roles.clone(),
        exp: expiration as usize,
    };
    let header = Header::new(Algorithm::HS512);
    let key = EncodingKey::from_secret(conf.env_vars().jwt_token_base.as_bytes());
    let jwt = encode(&header, &claims, &key);
    match jwt {
        Ok(x) => Ok(x),
        Err(_) => Err(JWTSecurityError::from("fail creating a token".to_string()).into()),
    }
}
