use super::{appstate::AppState, build_resp};
use crate::handlers::user_my_hd::get_user_id;
use actix_web::http::StatusCode;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use lib_licenses::services::assets::CreatableFildsAsset;
use lib_licenses::{
    errors::asset::{AssetDynamoDBError, AssetNoExistsError},
    services::assets::AssetManipulation,
};
use std::str::FromStr;
use uuid::Uuid;

pub async fn get_asset_by_token_id(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let asset_service = &state.asset_service;
    let id = path.into_inner();

    let uuid_id_op = Uuid::from_str(id.as_str());
    let token_id;
    match uuid_id_op {
        Err(e) => {
            return build_resp(e.to_string(), StatusCode::BAD_REQUEST);
        }
        Ok(v) => token_id = v,
    }

    let op_res = asset_service.get_by_id(&token_id).await;
    match op_res {
        Ok(asset) =>
        //HttpResponse::Ok().json(asset) ,
        {
            build_resp(serde_json::to_string(&asset).unwrap(), StatusCode::OK)
        }
        //build_resp( asset, StatusCode::OK),
        Err(e) => {
            if let Some(e) = e.downcast_ref::<AssetDynamoDBError>() {
                build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE)
            } else if let Some(_) = e.downcast_ref::<AssetNoExistsError>() {
                build_resp("".to_string(), StatusCode::NO_CONTENT)
            } else {
                build_resp("".to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

pub async fn get_all_my_assets(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
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

// #[derive(Debug, Serialize, Deserialize, Validate)]
// pub struct CreateAsset {
//     #[validate(length(max = 100))]
//     pub url: String,
//     #[validate(length(max = 100))]
//     pub license: String,

//     pub longitude: Option<f64>,
//     pub latitude: Option<f64>,
//}

pub async fn create_my_asset(
    req: HttpRequest,
    state: web::Data<AppState>,
    payload: web::Json<CreatableFildsAsset>,
) -> impl Responder {
    let asset_service = &state.asset_service;

    let user_id = get_user_id(&req);

    // let mut asset_fields: CreatableFildsAsset;

    // match payload.validate() {
    //     Err(e) => {
    //         return build_resp(e.to_string(), StatusCode::BAD_REQUEST);
    //     }
    //     Ok(_) => {
    //          asset_fields = CreatableFildsAsset {

    //          };
    //     }
    // }

    let op_res = asset_service.add(&payload, &user_id).await;

    match op_res {
        Err(e) => {
            if let Some(m) = e.downcast_ref::<AssetDynamoDBError>() {
                return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<AssetNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else {
                return build_resp("".to_string(), StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
        Ok(val) => build_resp(val.to_string(), StatusCode::OK),
    }
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct CreateNFT {
//     pub price: u64,
// }

// pub async fn create_my_nft(
//     req: HttpRequest,
//     state: web::Data<AppState>,
//     path: web::Path<String>,
//     payload: web::Json<CreateNFT>,
// ) -> impl Responder {
//     let user_service = &state.user_service;
//     let blockchain_service = &state.blockchain_service;

//     let user_id = get_user_id(&req);
//     let user_address;
//     let id = path.into_inner();

//     let uuid_id_op = Uuid::from_str(id.as_str());
//     let asset_id;
//     match uuid_id_op {
//         Err(_) => { return build_resp("no uuid as param".to_string(), StatusCode::BAD_REQUEST) },
//         Ok(v) => { asset_id = v.clone() },
//     }

//     let user_op =  user_service.get_by_user_id(&user_id).await;
//     match user_op {
//         Err(e) => {
//             if let Some(m) = e.downcast_ref::<UserDynamoDBError>() {
//                 return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
//             } else if let Some(m) = e.downcast_ref::<UserNoExistsError>() {
//                 return build_resp(m.to_string(), StatusCode::NO_CONTENT);
//             } else {
//                 return build_resp("unknown error finding the user".to_string(), StatusCode::INTERNAL_SERVER_ERROR);
//             }
//         }
//         Ok(user) => {
//             user_address = user.wallet_address().to_owned().unwrap();
//         },
//     }

//     let op_res = blockchain_service.add(
//         &asset_id,
//         &user_id,
//         &user_address,
//         &payload.price).await;

//     let transaction = match op_res {
//         Err(e) => {
//             if let Some(m) = e.downcast_ref::<AssetBlockachainError>() {
//                 return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
//             } else if let Some(m) = e.downcast_ref::<AssetDynamoDBError>() {
//                 return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
//             } else if let Some(m) = e.downcast_ref::<OwnerDynamoDBError>() {
//                 return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
//             } else if let Some(m) = e.downcast_ref::<AssetNoExistsError>() {
//                 return build_resp(m.to_string(), StatusCode::NO_CONTENT);
//             } else if let Some(m) = e.downcast_ref::<OwnerNoExistsError>() {
//                 return build_resp(m.to_string(), StatusCode::NO_CONTENT);
//             } else {
//                 return build_resp("unknonw error working with the blockchain".to_string(), StatusCode::INTERNAL_SERVER_ERROR);
//             }
//         },
//         Ok(tx) => tx,
//     };

//     return build_resp(transaction, StatusCode::OK);

// }
