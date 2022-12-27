use std::str::FromStr;

use actix_web::{web, Responder, HttpResponse, HttpRequest, http::header::TryIntoHeaderValue, dev::Payload };
use serde::{Deserialize, Serialize};

use crate::users::{
    models::user::{User},
    services::users::{UserManipulation }, errors::users::{DynamoDBError, UserAlreadyExistsError, UserNoExistsError},
};

use super::{appstate::AppState, jwt_middleware::UID_HEAD_KEY};

#[derive(Serialize,Deserialize)]
pub struct UpdateMyUser {
    pub wallet_address: Option<String>, 
    pub device: Option<String>,
    pub email: Option<String>
}
pub async fn update_my_user(req: HttpRequest,state: web::Data<AppState>, payload: web::Json<UpdateMyUser> ) -> impl Responder {
    let user_service = &state.user_service;

    let id = get_user_id(&req);

    let mut temp_user = User::new();
    if let Some(email) = &payload.email {
        temp_user.set_email(email);
    }
    if let Some(wallet) = &payload.wallet_address {
        temp_user.set_wallet_address(wallet);
    }
    if let Some(devc) = &payload.device {
        temp_user.set_device(devc);
    }

    let op_res = user_service.update_user(&id, &temp_user).await;
    match op_res {
        Err(e) => {
            if let Some(_) = e.downcast_ref::<DynamoDBError>() {
                HttpResponse::ServiceUnavailable().finish()    
            } else if  let Some(_) = e.downcast_ref::<UserNoExistsError>() {
                HttpResponse::BadRequest().finish()
            } else {
                HttpResponse::InternalServerError().finish()    
            }
        },
        Ok(_) => { 
            HttpResponse::Ok().finish()
        }
    }
}

pub async fn get_my_user(req: HttpRequest,state: web::Data<AppState>) -> impl Responder {
    let user_service = &state.user_service;
    
    let id = get_user_id(&req);

    let op_res = user_service.get_by_user_id(&id).await;
    match op_res {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => {
            if let Some(_) = e.downcast_ref::<DynamoDBError>() {
                HttpResponse::ServiceUnavailable().finish()   
            } else if  let Some(_) = e.downcast_ref::< UserNoExistsError>() {
                HttpResponse::NoContent().finish()
            } else {
                HttpResponse::InternalServerError().finish()    
            }
        }
    }
    
}

#[derive(Serialize,Deserialize)]
pub struct UpdatePasswordUser {
    pub password: String, 
}
pub async fn password_update_my_user( req: HttpRequest, state: web::Data<AppState>, payload: web::Json<UpdatePasswordUser> ) -> impl Responder {
    let user_service = &state.user_service;

    let id = get_user_id(&req);

    let op_res = user_service. update_password(&id, &payload.password).await;
    match op_res {
        Err(e) => {
            if let Some(_) = e.downcast_ref::<DynamoDBError>() {
                HttpResponse::ServiceUnavailable().finish()    
            } else if  let Some(_) = e.downcast_ref::<UserNoExistsError>() {
                HttpResponse::BadRequest().finish()
            } else {
                HttpResponse::InternalServerError().finish()    
            }
        },
        Ok(iid) => { 
            HttpResponse::Ok().finish()
        }
    }
}

fn get_user_id(req: &HttpRequest) -> String{
    let id = req.headers().get(UID_HEAD_KEY).unwrap();
    let id_aux = id.to_str().unwrap();
    let id_fin = String::from_str(id_aux).unwrap();
    return  id_fin;
}