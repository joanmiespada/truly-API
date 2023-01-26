
use lambda_http::RequestExt;
use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::Config;
use lib_licenses::errors::asset::{AssetDynamoDBError, AssetNoExistsError, AssetBlockachainError};
use lib_licenses::errors::owner::{OwnerDynamoDBError, OwnerNoExistsError};
use lib_licenses::models::asset::Asset;
use lib_licenses::services::assets::{AssetManipulation, AssetService};
use lib_licenses::services::owners::{OwnerService, OwnerManipulation };
use lib_users::services::users::{UsersService, UserManipulation};
use lib_users::errors::users::{UserNoExistsError, UserDynamoDBError};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use url::Url;
use uuid::Uuid;
use validator::{Validate, ValidationError};
use lib_licenses::services::nfts::{NFTsService, NFTsManipulation };

use crate::my_lambda::build_resp;

#[derive(Debug, Serialize, Validate, Deserialize)]
pub struct CreateNFT {
    //#[validate(length(max = 1000))]
    //pub hash: String,
    pub price: u64,
    //#[validate(length(max = 100))]
    //pub user_blockchain_address: String,
}
#[instrument]
pub async fn create_my_nft(
    req: &Request,
    _c: &Context,
    config: &Config,
    asset_service: &AssetService,
    owner_service: &OwnerService,
    blockchain_service: &NFTsService,
    user_service: &UsersService,
    asset_id: &Uuid,
    user_id: &String,
) -> Result<Response<String>, Box<dyn std::error::Error + Send+ Sync >> {
 
    let price: u64;
    let hash:String;
    let user_address:String;
    
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
                }
            },
        },
    }
 
    let hash_op = asset_service.get_by_id(asset_id).await;

    match hash_op {
        Err(e) => {
            if let Some(m) = e.downcast_ref::<AssetDynamoDBError>() {
                return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<AssetNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else if let Some(m) = e.downcast_ref::<ValidationError>() {
                return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
            } else {
                return build_resp("unknown error finding the asset to be minted".to_string(), StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
        Ok(asset) => {
            hash = asset.hash().to_owned().unwrap()
        },
    }
  
    let user_op =  user_service.get_by_user_id(user_id).await;
    match user_op {
        Err(e) => {
            if let Some(m) = e.downcast_ref::<UserDynamoDBError>() {
                return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<UserNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else {
                return build_resp("unknown error finding the user".to_string(), StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
        Ok(user) => {
            user_address = user.wallet_address().to_owned().unwrap();
        },
    }
   
 
    let owner_op = owner_service.get_by_user_asset_ids(asset_id, user_id).await;

    match owner_op {
        Err(e) => {
            if let Some(m) = e.downcast_ref::<OwnerDynamoDBError>() {
                return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<OwnerNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else {
                return build_resp("unknown error finding the ownership between user and asset".to_string(), StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
        Ok(_) => {},
    }

    let op_res = blockchain_service.add(asset_id, &user_address, &hash, &price).await;
    
    let transaction = match op_res {
        Err(e) => {
            if let Some(m) = e.downcast_ref::<AssetBlockachainError>() {
                return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else {
                return build_resp("unknonw error working with the blockchain".to_string(), StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
        Ok(tx) => tx,
    };
    
    let stamp_op = asset_service.minted(asset_id, &transaction).await;
    match stamp_op  {
       Err(e) => {
            //TODO! implement dead queue letter!!!!
            if let Some(m) = e.downcast_ref::<AssetDynamoDBError>() {
                return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else {
                return build_resp("unknonw error when storing the tx on asset".to_string(), StatusCode::INTERNAL_SERVER_ERROR);
            }
       },
       Ok(_)=> {} 
        
    }
    
    let transaction = "".to_string();
    return build_resp(transaction, StatusCode::OK);



}
