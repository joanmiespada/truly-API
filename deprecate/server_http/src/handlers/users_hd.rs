use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use lib_users::{
    errors::users::{
        UserAlreadyExistsError, UserDynamoDBError, UserMismatchError, UserNoExistsError,
    },
    models::user::User,
    services::users::{PromoteUser, UpdatableFildsUser, UserManipulation},
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
        Ok(vec_user) => HttpResponse::Ok().json(vec_user),
    }
}

pub async fn get_user_by_id(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let user_service = &state.user_service;
    let id = path.into_inner();
    let op_res = user_service.get_by_id(&id).await;
    match op_res {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => {
            if let Some(_) = e.downcast_ref::<UserDynamoDBError>() {
                HttpResponse::ServiceUnavailable().finish()
            } else if let Some(_) = e.downcast_ref::<UserNoExistsError>() {
                HttpResponse::NoContent().finish()
            } else {
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}

#[derive(Deserialize)]
pub struct NewUser {
    pub wallet_address: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub device: Option<String>,
}

#[derive(Serialize)]
pub struct NewIdUser {
    pub id: String,
}

pub async fn add_user(state: web::Data<AppState>, payload: web::Json<NewUser>) -> impl Responder {
    let user_service = &state.user_service;

    let mut user = User::new();

    if let Some(eml) = &payload.email {
        user.set_email(eml);
    }

    if let Some(dvc) = &payload.device {
        user.set_device(dvc);
    }

    let op_res = user_service.add(&mut user, &payload.password).await;
    match op_res {
        Err(e) => {
            if let Some(err) = e.downcast_ref::<UserDynamoDBError>() {
                HttpResponse::ServiceUnavailable().body(err.to_string())
            } else if let Some(err) = e.downcast_ref::<UserAlreadyExistsError>() {
                HttpResponse::BadRequest().body(err.to_string())
            } else if let Some(err) = e.downcast_ref::<UserMismatchError>() {
                HttpResponse::BadRequest().body(err.to_string())
            } else {
                HttpResponse::InternalServerError().finish()
            }
        }
        Ok(iid) => {
            let res = NewIdUser { id: iid };
            HttpResponse::Ok().json(res)
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateUser {
    pub device: Option<String>,
    pub email: Option<String>,
    pub status: Option<String>,
}
pub async fn update_user(
    state: web::Data<AppState>,
    payload: web::Json<UpdateUser>,
    path: web::Path<String>,
) -> impl Responder {
    let id = path.into_inner();
    _update_user(state, payload, &id).await
}

pub async fn _update_user(
    state: web::Data<AppState>,
    payload: web::Json<UpdateUser>,
    id: &String,
) -> impl Responder {
    let user_service = &state.user_service;

    let user_fields = UpdatableFildsUser {
        device: payload.device.clone(),
        email: payload.email.clone(),
        status: payload.status.clone(),
        wallet: None,
    };

    let op_res = user_service.update(&id, &user_fields).await;
    match op_res {
        Err(e) => {
            if let Some(_) = e.downcast_ref::<UserDynamoDBError>() {
                HttpResponse::ServiceUnavailable().finish()
            } else if let Some(_) = e.downcast_ref::<UserNoExistsError>() {
                HttpResponse::BadRequest().finish()
            } else {
                HttpResponse::InternalServerError().finish()
            }
        }
        Ok(_) => HttpResponse::Ok().finish(),
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdatePasswordUser {
    pub password: String,
}
pub async fn password_update_user(
    state: web::Data<AppState>,
    payload: web::Json<UpdatePasswordUser>,
    path: web::Path<String>,
) -> impl Responder {
    let user_service = &state.user_service;

    let id = path.into_inner();

    let op_res = user_service.update_password(&id, &payload.password).await;
    match op_res {
        Err(e) => {
            if let Some(_) = e.downcast_ref::<UserDynamoDBError>() {
                HttpResponse::ServiceUnavailable().finish()
            } else if let Some(_) = e.downcast_ref::<UserNoExistsError>() {
                HttpResponse::BadRequest().finish()
            } else {
                HttpResponse::InternalServerError().finish()
            }
        }
        Ok(_) => HttpResponse::Ok().finish(),
    }
}

pub async fn promote_user(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let user_service = &state.user_service;

    let id = path.into_inner();

    let op_res = user_service
        .promote_user_to(&id, &PromoteUser::Upgrade)
        .await;
    match op_res {
        Err(e) => {
            if let Some(_) = e.downcast_ref::<UserDynamoDBError>() {
                HttpResponse::ServiceUnavailable().finish()
            } else if let Some(_) = e.downcast_ref::<UserNoExistsError>() {
                HttpResponse::BadRequest().finish()
            } else {
                HttpResponse::InternalServerError().finish()
            }
        }
        Ok(_) => HttpResponse::Ok().finish(),
    }
}

pub async fn downgrade_user(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let user_service = &state.user_service;

    let id = path.into_inner();

    let op_res = user_service
        .promote_user_to(&id, &PromoteUser::Downgrade)
        .await;
    match op_res {
        Err(e) => {
            if let Some(_) = e.downcast_ref::<UserDynamoDBError>() {
                HttpResponse::ServiceUnavailable().finish()
            } else if let Some(_) = e.downcast_ref::<UserNoExistsError>() {
                HttpResponse::BadRequest().finish()
            } else {
                HttpResponse::InternalServerError().finish()
            }
        }
        Ok(_) => HttpResponse::Ok().finish(),
    }
}

/*
pub fn delete_user(state: web::Data<AppState>) -> impl Responder {
    HttpResponse::MethodNotAllowed().finish()
}*/
