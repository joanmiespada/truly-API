
use lambda_http::RequestExt;
use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::config::Config;
use lib_licenses::errors::asset::{AssetDynamoDBError, AssetNoExistsError, AssetBlockachainError};
use lib_blockchain::errors::nft::NftUserAddressMalformedError;
use lib_licenses::errors::owner::{OwnerDynamoDBError, OwnerNoExistsError};
use lib_licenses::services::assets::AssetService;
use lib_licenses::services::owners::OwnerService;
use lib_users::services::users::UsersService;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate};
use lib_blockchain::services::nfts::{NFTsService, NFTsManipulation };

use crate::my_lambda::build_resp;

#[derive(Debug, Serialize, Validate, Deserialize, Clone, Copy)]
pub struct CreateNFT {
    //#[validate(length(max = 1000))]
    //pub hash: String,
    pub price: u64,
    pub asset_id: Uuid
    //#[validate(length(max = 100))]
    //pub user_blockchain_address: String,
}
#[tracing::instrument]
pub async fn create_my_nft(
    req: &Request,
    _c: &Context,
    config: &Config,
    asset_service: &AssetService,
    owner_service: &OwnerService,
    blockchain_service: &NFTsService,
    user_service: &UsersService,
    user_id: &String,
) -> Result<Response<String>, Box<dyn std::error::Error + Send+ Sync >> {
 
    let price: u64;
    let asset_id: Uuid;
    
    match req.payload::<CreateNFT>() {
        Err(e) => {
            return build_resp(e.to_string(), StatusCode::BAD_REQUEST);
        }
        Ok(op_payload) => match op_payload {
            None => {
                return build_resp("no payload found".to_string(), StatusCode::BAD_REQUEST);
            }
            Some(payload) => match payload.validate() {
                Err(e) => {
                    return build_resp(e.to_string(), StatusCode::BAD_REQUEST);
                }
                Ok(_) => {
                    price = payload.price;
                    asset_id = payload.asset_id;
                }
            },
        },
    }

    let op_res = blockchain_service.try_mint(
        &asset_id, 
        user_id, 
        &price).await;
    
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
                return build_resp("unknonw error working with the blockchain".to_string(), StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
        Ok(tx) => format!("{}", tx)
        //result().clone().unwrap(),
    };
    
    return build_resp(transaction, StatusCode::OK);

}
