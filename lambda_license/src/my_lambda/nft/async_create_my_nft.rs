
use std::str::FromStr;

use lambda_http::RequestExt;
use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_async_ops::errors::AsyncOpError;
use lib_async_ops::{SQSMessage, send};
use lib_config::config::Config;
use lib_licenses::services::assets::AssetService;
use lib_licenses::services::owners::OwnerService;
use lib_users::services::users::UsersService;
use url::Url;
use uuid::Uuid;
use validator::Validate;
use lib_licenses::services::nfts::{NFTsService, CreateNFTAsync };

use crate::my_lambda::build_resp;

use super::create_my_nft::CreateNFT;

#[tracing::instrument]
pub async fn async_create_my_nft(
    req: &Request,
    _c: &Context,
    config: &Config,
    asset_service: &AssetService,
    owner_service: &OwnerService,
    blockchain_service: &NFTsService,
    user_service: &UsersService,
    user_id: &String,
) -> Result<Response<String>, Box<dyn std::error::Error + Send+ Sync >> {
 
    let new_nft; 
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
                    new_nft = payload.clone();
                }
            },
        },
    }

    let new_nft_async = CreateNFTAsync {
        user_id: user_id.clone(),
        asset_id: new_nft.asset_id,
        price: new_nft.price
    };

    //let queue_url = find(&client).await?;

    let json_text = serde_json::to_string(&new_nft_async)?;

    let message = SQSMessage {
        id: Uuid::new_v4().to_string(),
        body:  json_text.to_owned(),
        //group: "MyGroup".to_owned(),
    };
   
    let url = config.env_vars().queue_mint_async().to_owned();
    let queue_mint_id = Url::from_str(&url).unwrap();

    let enqueded_op = send( config, &message, queue_mint_id).await;

    
    let message = match enqueded_op {
        Err(e) => {
            if let Some(m) = e.downcast_ref::< AsyncOpError>() {
                return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else {
                return build_resp("unknonw error working with the blockchain".to_string(), StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
        Ok(val)=> val
    };
        
    
    return build_resp(message, StatusCode::OK);

}
