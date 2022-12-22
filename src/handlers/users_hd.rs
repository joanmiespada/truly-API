
use actix_web::{web, Responder, HttpResponse };
use serde::{Deserialize, Serialize};

use crate::users::{
    models::user::{User},
    services::users::{UserManipulation }, errors::users::{DynamoDBError, UserAlreadyExistsError, UserNoExistsError},
};

use super::appstate::AppState;

const PAGESIZE_MAX: u32 = 20;
const PAGESIZE_MIN: u32 = 5;
const PAGENUM_MIN: u32 = 1;



#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct QueryPagination {
    pageNumber: Option<u32>,
    pageSize: Option<u32>,
}

pub async fn get_users(
    state: web::Data<AppState>,
    params: web::Query<QueryPagination>,
) -> impl Responder {
    let user_service = &state.user_service;

    let page_num = params.pageNumber.unwrap_or(PAGENUM_MIN);
    let page_size = params.pageSize.unwrap_or(PAGESIZE_MAX);

    if page_num < PAGENUM_MIN {
        let message = format!("pageNumber is {}, when the minimum value is 1", page_num);
        return HttpResponse::BadRequest().body(message);
    }
    if page_size < PAGESIZE_MIN || page_size > PAGESIZE_MAX {
        let message = format!(
            "pageSize is {0}, but it must be between {1} and {2}",
            page_size, PAGENUM_MIN, PAGESIZE_MAX
        );
        return HttpResponse::BadRequest().body(message);
    }
    let res = user_service.get_all(page_num, page_size).await;
    match res {
        Err(_) => HttpResponse::InternalServerError().finish(), 
        Ok(vec_user)=> HttpResponse::Ok().json(vec_user)
    }
}

pub async fn get_user_by_id(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let user_service = &state.user_service;
    let id = path.into_inner();
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

#[derive(Deserialize)]
pub struct NewUser {
    pub wallet_address: String,
    pub email: String,
    pub password: String,
    pub device: String,
}

#[derive(Serialize)]
pub struct NewIdUser {
    pub id: String
}

pub async fn add_user(state: web::Data<AppState>, payload: web::Json<NewUser>) -> impl Responder {
    let user_service = &state.user_service;

    let mut user = User::new();
    user.set_email(&payload.email);
    user.set_wallet_address(&payload.wallet_address);
    user.set_device(&payload.device);

    let op_res = user_service.add_user(&mut user, &payload.password).await;
    match op_res {
        Err(e) => {
            if let Some(_) = e.downcast_ref::<DynamoDBError>() {
                HttpResponse::ServiceUnavailable().finish()    
            } else if  let Some(_) = e.downcast_ref::<UserAlreadyExistsError>() {
                HttpResponse::BadRequest().finish()
            } else {
                HttpResponse::InternalServerError().finish()    
            }
        },
        Ok(iid) => { 
            let res= NewIdUser{ id: iid};
            HttpResponse::Ok().json(res)
        }
    }

    //format!("{{'id':'{}'}}", new_id)
}

/* 
#[derive(Serialize,Deserialize)]
pub struct Filter {
    pub field: String,
    pub value: String
}
pub async fn get_user_by_filter(state: web::Data<AppState>, payload: web::Json<Filter>) -> impl Responder {
    let user_service = &state.user_service;
    let op_res = user_service.get_by_filter(&payload.field, &payload.value ).await;
    match op_res {
        Ok(users) => {
            if users.len() == 0 {
                HttpResponse::NoContent().finish()
            }else{
                 HttpResponse::Ok().json(users)    
            }
        },
        Err(e) => {
            if let Some(_) = e.downcast_ref::<DynamoDBError>() {
                HttpResponse::ServiceUnavailable().finish()    
            } else {
                HttpResponse::InternalServerError().finish()    
            }
        }
    }
    
}
*/

#[derive(Serialize,Deserialize)]
pub struct UpdateUser {
    pub wallet_address: String, 
    pub device: String,
    pub email: String
}
pub async fn update_user(state: web::Data<AppState>, payload: web::Json<UpdateUser>, path: web::Path<String>) -> impl Responder {
    let user_service = &state.user_service;

    let id = path.into_inner();

    let mut user = User::new();
    user.set_email(&payload.email);
    user.set_wallet_address(&payload.wallet_address);
    user.set_device(&payload.device);

    let op_res = user_service.update_user(&id, &user).await;
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
            HttpResponse::Ok().body(iid.to_string())
        }
    }
}

//TODO---> JWT to identify the current user, only Admins can perform it!!!
pub async fn promote_user(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let user_service = &state.user_service;

    let id = path.into_inner();

    let op_res = user_service.promote_user_to_admin(&id).await;
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
            HttpResponse::Ok().body(iid.to_string())
        }
    }
}

pub fn delete_user(state: web::Data<AppState>) -> impl Responder {
    HttpResponse::MethodNotAllowed().finish()
}
 