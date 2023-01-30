use std::{fmt, str::FromStr};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::repositories::ganache::{GanacheRepo, NFTsRepository};
use crate::services::assets::{AssetService, AssetManipulation};
use crate::services::owners::{OwnerService, OwnerManipulation};

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[async_trait]
pub trait NFTsManipulation {
    async fn add(
        &self,
        asset_id: &Uuid,
        user_id: &String,
        user_address: &String,
        asset_service: &AssetService,
        owner_service: &OwnerService,
        price: &u64,
    ) -> ResultE<String>;
    async fn get(&self, asset_id: &Uuid) -> ResultE<NTFContentInfo>;
}

#[derive(Debug)]
pub struct NFTsService(GanacheRepo);

impl NFTsService {
    pub fn new(repo: GanacheRepo) -> NFTsService {
        NFTsService(repo)
    }
}

#[async_trait]
impl NFTsManipulation for NFTsService {
    #[tracing::instrument()]
    async fn add(
        &self,
        asset_id: &Uuid,
        user_id: &String,
        user_wallet_address: &String,
        asset_service: &AssetService,
        owner_service: &OwnerService,
        price: &u64,
    ) -> ResultE<String> {
        //let hash_file;

        let asset = asset_service.get_by_id(asset_id).await?;
        let hash_file = asset.hash().to_owned().unwrap();

        // match hash_op {
        //     Err(e) => {
        //         return Err(e);

        //         // if let Some(m) = e.downcast_ref::<AssetDynamoDBError>() {
        //         //     return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
        //         // } else if let Some(m) = e.downcast_ref::<AssetNoExistsError>() {
        //         //     return build_resp(m.to_string(), StatusCode::NO_CONTENT);
        //         // } else if let Some(m) = e.downcast_ref::<ValidationError>() {
        //         //     return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
        //         // } else {
        //         //     return build_resp(
        //         //         "unknown error finding the asset to be minted".to_string(),
        //         //         StatusCode::INTERNAL_SERVER_ERROR,
        //         //     );
        //         // }
        //     }
        //     Ok(asset) => hash_file = asset.hash().to_owned().unwrap(),
        // }

        owner_service
            .get_by_user_asset_ids(asset_id, user_id)
            .await?;

        // match owner_op {
        //     Err(e) => {
        //         return Err(e);
        //         // if let Some(m) = e.downcast_ref::<OwnerDynamoDBError>() {
        //         //     return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
        //         // } else if let Some(m) = e.downcast_ref::<OwnerNoExistsError>() {
        //         //     return build_resp(m.to_string(), StatusCode::NO_CONTENT);
        //         // } else {
        //         //     return build_resp(
        //         //         "unknown error finding the ownership between user and asset".to_string(),
        //         //         StatusCode::INTERNAL_SERVER_ERROR,
        //         //     );
        //         // }
        //     }
        //     Ok(_) => {}
        // }

        let transaction = self
            .0
            .add(asset_id, user_wallet_address, &hash_file, price)
            .await?;

        // let transaction = match op_res {
        //     Err(e) => {
        //         if let Some(m) = e.downcast_ref::<AssetBlockachainError>() {
        //             return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
        //         } else {
        //             return build_resp(
        //                 "unknonw error working with the blockchain".to_string(),
        //                 StatusCode::INTERNAL_SERVER_ERROR,
        //             );
        //         }
        //     }
        //     Ok(tx) => tx,
        // };

        let stamp_op = asset_service.minted(asset_id, &transaction).await;
        match stamp_op {
            Err(e) => {
                //TODO! implement dead queue letter!!!!
                return Err(e);
                // if let Some(m) = e.downcast_ref::<AssetDynamoDBError>() {
                //     return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
                // } else {
                //     return build_resp(
                //         "unknonw error when storing the tx on asset, after minting it".to_string(),
                //         StatusCode::INTERNAL_SERVER_ERROR,
                //     );
                // }
            }
            Ok(_) => {} //Ok(aux)
        }
        Ok(transaction)
    }

    #[tracing::instrument()]
    async fn get(&self, asset_id: &Uuid) -> ResultE<NTFContentInfo> {
        let aux = self.0.get(asset_id).await?;
        let res = NTFContentInfo {
            hash_file: aux.hashFile,
            uri: aux.uri,
            price: aux.price,
            state: NTFState::from_str(&aux.state.to_string()).unwrap(),
        };
        Ok(res)
    }
}

impl Clone for NFTsService {
    #[tracing::instrument()]
    fn clone(&self) -> NFTsService {
        let aux = NFTsService {
            0: self.0.clone(),
        };
        return aux;
    }
}


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NTFContentInfo {
    pub hash_file: String,
    pub uri: String,
    pub price: u64,
    pub state: NTFState,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NTFState {
    Active,
    Inactive,
}
impl fmt::Debug for NTFState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Inactive => write!(f, "Inactive"),
        }
    }
}
impl fmt::Display for NTFState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Inactive => write!(f, "Inactive"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseNTFStateError;
impl std::str::FromStr for NTFState {
    type Err = ParseNTFStateError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "Active" => Ok(NTFState::Active),
            "Inactive" => Ok(NTFState::Inactive),
            _ => Err(ParseNTFStateError),
        }
    }
}
