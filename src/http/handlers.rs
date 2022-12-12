

use actix_web::{Responder, web};
use serde::Deserialize;

use crate::users::services::users::{UsersService, UserManipulation};


pub struct AppState {
    pub user_service: UsersService
}


#[derive(Deserialize)]
pub struct Info {
    username: String,
}
// use serde::Deserialize;
pub async fn get_users(data: web::Data<AppState>) -> impl Responder {
    let userService = &data.user_service;
    let res = userService.get_all();
    format!("hello from get users",)
}

pub async fn get_user_by_id() -> impl Responder {
    format!("hello from get users by id")
}

pub async fn add_user() -> impl Responder {
    format!("hello from add user")
}



/* 
pub async fn delete_user() -> impl Responder {
    format!("hello from delete user")
}*/