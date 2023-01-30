use std::str::FromStr;

use actix_web::{web, HttpResponse, Responder, HttpRequest};
use uuid::Uuid;
use crate::handlers::user_my_hd::get_user_id;
use super::appstate::AppState;
use lib_licenses::{services::assets::{AssetManipulation}, errors::asset::{AssetDynamoDBError, AssetNoExistsError}};



pub async fn get_asset_by_token_id(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let asset_service = &state.asset_service;
    let id = path.into_inner();

    let uuid_id_op = Uuid::from_str(id.as_str() );
    let token_id;
    match uuid_id_op {
        Err(_)=> {return HttpResponse::BadRequest().finish(); },
        Ok(v)=> token_id=v
    }

    let op_res = asset_service.get_by_id(&token_id).await;
    match op_res {
        Ok(token) => HttpResponse::Ok().json(token),
        Err(e) => {
            if let Some(_) = e.downcast_ref::<AssetDynamoDBError>() {
                HttpResponse::ServiceUnavailable().finish()
            } else if let Some(_) = e.downcast_ref::<AssetNoExistsError>() {
                HttpResponse::NoContent().finish()
            } else {
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}


pub async fn get_all_my_assets(req: HttpRequest,state: web::Data<AppState>, _path: web::Path<String>) -> impl Responder {

    let asset_service = &state.asset_service;

    let user_id = get_user_id(&req);
    
    let op_res = asset_service.get_by_user_id(&user_id).await;
    match op_res {
        Ok(token) => HttpResponse::Ok().json(token),
        Err(e) => {
            if let Some(_) = e.downcast_ref::<AssetDynamoDBError>() {
                HttpResponse::ServiceUnavailable().finish()
            } else if let Some(_) = e.downcast_ref::<AssetNoExistsError>() {
                HttpResponse::NoContent().finish()
            } else {
                HttpResponse::InternalServerError().finish()
            }
        }
    }


}