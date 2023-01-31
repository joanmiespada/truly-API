use std::str::FromStr;
use crate::handlers::appstate::AppState;
use actix_web:: {http::StatusCode, web ,Responder, HttpRequest };
use lib_licenses::errors::asset::{AssetBlockachainError, AssetDynamoDBError, AssetNoExistsError};
use lib_licenses::errors::nft::NftUserAddressMalformedError;
use lib_licenses::errors::owner::{OwnerDynamoDBError, OwnerNoExistsError};
use lib_licenses::services::nfts::NFTsManipulation;
use lib_users::errors::users::{UserDynamoDBError, UserNoExistsError};
use lib_users::services::users::UserManipulation;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::handlers::build_resp;
use crate::handlers::user_my_hd::get_user_id;

#[derive(Debug, Serialize, Deserialize)]
pub struct NewNFT {
    pub price: u64,
    pub asset_id: Uuid
}

pub async fn add_nft(req: HttpRequest,state: web::Data<AppState>, payload: web::Json<NewNFT> ) -> impl Responder {
    let user_service = &state.user_service;
    let blockchain_service = &state.blockchain_service;

    let user_address: String;
    let user_id = get_user_id(&req);
    //let id = path.into_inner();
    // let asset_id_op =  uuid::Uuid::from_str(id.as_str() );
    // let asset_id;
    
    // match asset_id_op {
    //     Err(e) => {
    //         return build_resp(e.to_string(), StatusCode::BAD_REQUEST);
    //     }
    //     Ok(aidd) => {
    //         asset_id = aidd.clone();
    //     }
    // }
    

    let user_op = user_service.get_by_user_id(&user_id).await;
    match user_op {
        Err(e) => {
            if let Some(m) = e.downcast_ref::<UserDynamoDBError>() {
                return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<UserNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);

            } else {
                 return build_resp(
                     "unknown error finding the user".to_string(),
                     StatusCode::INTERNAL_SERVER_ERROR,
                 );
            }
        }
        Ok(user) => {
            user_address = user.wallet_address().to_owned().unwrap();
        }
    }

    let op_res = blockchain_service
        .add(
            &payload.asset_id,
            &user_id,
            &user_address,
            &payload.price,
        )
        .await;

    let transaction = match op_res {
        Err(e) => {
            if let Some(m) = e.downcast_ref::<AssetBlockachainError>() {
                return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<AssetDynamoDBError>() {
                return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<OwnerDynamoDBError>() {
                return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<AssetNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else if let Some(m) = e.downcast_ref::<OwnerNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else if let Some(m) = e.downcast_ref::<NftUserAddressMalformedError>() {
                return build_resp(m.to_string(), StatusCode::NOT_ACCEPTABLE);
            } else {
                return build_resp(
                    "unknonw error working with the blockchain".to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                );
            }
        }
        Ok(tx) => tx,
    };

    return build_resp(transaction, StatusCode::OK);
}
