use crate::handlers::appstate::AppState;
use crate::handlers::build_resp;
use crate::handlers::user_my_hd::get_user_id;
use actix_web::{http::StatusCode, web, HttpRequest, Responder};
use lib_blockchain::errors::nft::NftUserAddressMalformedError;
use lib_blockchain::services::nfts::NFTsManipulation;
use lib_licenses::errors::asset::{AssetBlockachainError, AssetDynamoDBError, AssetNoExistsError};
use lib_licenses::errors::owner::{OwnerDynamoDBError, OwnerNoExistsError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct NewNFT {
    pub price: u64,
    pub asset_id: Uuid,
}

pub async fn add_nft(
    req: HttpRequest,
    state: web::Data<AppState>,
    payload: web::Json<NewNFT>,
) -> impl Responder {
    //let user_service = &state.user_service;
    let blockchain_service = &state.blockchain_service;

    //let user_address: String;
    let user_id = get_user_id(&req);

    let op_res = blockchain_service
        .try_mint(&payload.asset_id, &user_id, &Some(payload.price))
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
        Ok(tx) => format!("{}", tx),
    };

    return build_resp(transaction, StatusCode::OK);
}
