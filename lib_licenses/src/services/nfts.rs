use std::{fmt, str::FromStr};

use async_trait::async_trait;
use lib_async_ops::{SQSMessage, send as send_async_message};
use lib_config::config::Config;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::models::asset::MintingStatus;
use crate::repositories::ganache::{GanacheRepo, NFTsRepository};
use crate::repositories::keypairs::{KeyPairRepo, KeyPairRepository};
use crate::services::assets::{AssetManipulation, AssetService};
use crate::services::owners::{OwnerManipulation, OwnerService};

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[async_trait]
pub trait NFTsManipulation {
    async fn try_mint(&self, asset_id: &Uuid, user_id: &String, price: &u64) -> ResultE<String>;
    async fn get(&self, asset_id: &Uuid) -> ResultE<NTFContentInfo>;
    //async fn create_account()->   ResultE<(String, String, String)>;
}

#[derive(Debug)]
pub struct NFTsService {
    blockchain: GanacheRepo,
    keys_repo: KeyPairRepo,
    asset_service: AssetService,
    owner_service: OwnerService,
    config: Config,
}

impl NFTsService {
    pub fn new(
        repo: GanacheRepo,
        keys_repo: KeyPairRepo,
        asset_service: AssetService,
        owner_service: OwnerService,
        config: Config,
    ) -> NFTsService {
        NFTsService {
            blockchain: repo,
            keys_repo: keys_repo,
            asset_service: asset_service,
            owner_service: owner_service,
            config,
        }
    }
}

#[async_trait]
impl NFTsManipulation for NFTsService {
    #[tracing::instrument()]
    async fn try_mint(&self, asset_id: &Uuid, user_id: &String, price: &u64) -> ResultE<String> {
        let asset = self.asset_service.get_by_id(asset_id).await?;
        let hash_file = asset.hash().to_owned().unwrap();

        self.owner_service
            .get_by_user_asset_ids(asset_id, user_id)
            .await?;

        let user_wallet_address = self.keys_repo.get_or_create(user_id).await?;

        self.asset_service
            .mint_status(asset_id, &None, MintingStatus::Started)
            .await?;

        let transaction_op = self
            .blockchain
            .add(asset_id, &user_wallet_address, &hash_file, price)
            .await;

        match transaction_op {
            Err(e) => {
                let url = self.config.env_vars().dead_letter_queue_mint().to_owned();
                let queue_mint_errors_id = Url::from_str(&url).unwrap();

                let message = SQSMessage {
                    id: Uuid::new_v4().to_string(),
                    body: format!(
                        "error minting asset id: {} with user id: {}",
                        asset_id, user_id
                    ),
                };
                send_async_message( &self.config, &message, queue_mint_errors_id).await?;

                self.asset_service
                    .mint_status(asset_id, &Some(e.to_string()), MintingStatus::Error)
                    .await?;

                return Err(e.into());
            }
            Ok(transaction) => {
                self.asset_service
                    .mint_status(
                        asset_id,
                        &Some(transaction.clone()),
                        MintingStatus::CompletedSuccessfully,
                    )
                    .await?;
                Ok(transaction)
            }
        }
    }

    #[tracing::instrument()]
    async fn get(&self, asset_id: &Uuid) -> ResultE<NTFContentInfo> {
        let aux = self.blockchain.get(asset_id).await?;
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
            blockchain: self.blockchain.clone(),
            keys_repo: self.keys_repo.clone(),
            owner_service: self.owner_service.clone(),
            asset_service: self.asset_service.clone(),
            config: self.config.clone()
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateNFTAsync {
    pub price: u64,
    pub asset_id: Uuid,
    pub user_id: String,
}
