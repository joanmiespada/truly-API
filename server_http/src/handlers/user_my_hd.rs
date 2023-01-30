use std::str::FromStr;

use actix_web::{web, Responder, HttpResponse, HttpRequest  };
use serde::{Deserialize, Serialize};

use lib_users::{
    services::users::{UserManipulation }, errors::users::{UserDynamoDBError,  UserNoExistsError},
};

use super::{appstate::AppState, jwt_middleware::UID_HEAD_KEY, users_hd::{UpdateUser, _update_user}};

pub async fn update_my_user(req: HttpRequest,state: web::Data<AppState>, payload: web::Json<UpdateUser> ) -> impl Responder {

    let id = get_user_id(&req);
    _update_user(state, payload, &id).await
}

pub async fn get_my_user(req: HttpRequest,state: web::Data<AppState>) -> impl Responder {
    let user_service = &state.user_service;
    
    let id = get_user_id(&req);

    let op_res = user_service.get_by_user_id(&id).await;
    match op_res {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => {
            if let Some(_) = e.downcast_ref::<UserDynamoDBError>() {
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
            if let Some(_) = e.downcast_ref::<UserDynamoDBError>() {
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

pub fn get_user_id(req: &HttpRequest) -> String{
    let id = req.headers().get(UID_HEAD_KEY).unwrap();
    let id_aux = id.to_str().unwrap();
    let id_fin = String::from_str(id_aux).unwrap();
    return  id_fin;
}