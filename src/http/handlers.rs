use actix_web::{web, Responder };
use serde::Deserialize;

use crate::users::services::users::{UserManipulation, UsersService};

const PAGESIZE_MAX: u32 = 20;
const PAGESIZE_MIN: u32 = 5;
const PAGENUM_MIN: u32 = 1;

pub struct AppState {
    pub user_service: UsersService,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct QueryPagination {
    pageNumber: Option<u32>,
    pageSize: Option<u32>,
}

// use serde::Deserialize;
pub async fn get_users(
    state: web::Data<AppState>,
    params: web::Query<QueryPagination>,
) -> impl Responder {
    let user_service = &state.user_service;

    let page_num = params.pageNumber.unwrap_or(PAGENUM_MIN);
    let page_size = params.pageSize.unwrap_or(PAGESIZE_MAX);

    if page_num < PAGENUM_MIN {
        return format!(
            "pageNumber is {}, when the minimum value is 1",
            page_num
        );
    }
    if page_size < PAGESIZE_MIN || page_size > PAGESIZE_MAX {
        return format!(
            "pageSize is {0}, but it must be between {1} and {2}",
            page_size, PAGENUM_MIN, PAGESIZE_MAX
        );
    }
    let res = user_service
        .get_all(page_num, page_size)
        .await;
    let serialized_users = serde_json::to_string(&res).unwrap();
    format!("{}", serialized_users)
}

pub async fn get_user_by_id(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let user_service = &state.user_service;
    let id = path.into_inner();
    let user_nullable = user_service.get_by_user_id(id).await;
    let user = user_nullable.unwrap();
    let serialized_user = serde_json::to_string(&user).unwrap();
    format!("{}", serialized_user)
}

#[derive(Deserialize)]
pub struct NewUser {}

pub async fn add_user() -> impl Responder {
    let new_id = 0;
    format!("{}", new_id)
}

/*
pub async fn delete_user() -> impl Responder {
    format!("hello from delete user")
}*/
