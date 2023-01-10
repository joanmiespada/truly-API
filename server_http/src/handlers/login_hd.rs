use actix_web::{web, HttpResponse, Responder};
use lib_util_jwt::create_jwt;
use serde::{Deserialize, Serialize};
use lib_users::{
        errors::users::{DynamoDBError, UserNoExistsError},
};

use super::{appstate::AppState};
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
            let roles: Vec<String> = log_inf.roles.into_iter().map(|ur| ur.to_string()).collect();
            let token_secret = conf.env_vars().jwt_token_base();
            let token_creation_ops = create_jwt(&log_inf.user_id, &roles, token_secret);
            match token_creation_ops {
                Err(_) => HttpResponse::InternalServerError().finish(),
                Ok(token) => HttpResponse::Ok().json(&LoginResponse { token }),
            }
        }
    }
}

