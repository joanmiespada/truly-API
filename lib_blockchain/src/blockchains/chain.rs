use std::fmt;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::block_tx::BlockchainTx;
use crate::models::keypair::KeyPair;

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[async_trait]
pub trait NFTsRepository: Send + Sync + CloneBoxNFTsRepository {
    async fn add(
        &self,
        asset_id: &Uuid,
        user_key: &KeyPair,
        hash_file: &String,
        hash_algorithm: &String,
        price: &Option<u64>,
        counter: &u64,
    ) -> ResultE<BlockchainTx>;

    //async fn get(&self, asset_id: &Uuid) -> ResultE<ContractContentInfo>;
    async fn get(&self, token: &String) -> ResultE<ContractContentInfo>;
    fn contract_id(&self) -> u16;
}

impl fmt::Debug for dyn NFTsRepository + Sync + Send {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Blockchain")
    }
}

pub trait CloneBoxNFTsRepository {
    fn clone_box(&self) -> Box<dyn NFTsRepository + Sync + Send>;
}

impl<T> CloneBoxNFTsRepository for T
where
    T: 'static + NFTsRepository + Clone + Send + Sync,
{
    fn clone_box(&self) -> Box<dyn NFTsRepository + Send + Sync> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn NFTsRepository + Send + Sync> {
    fn clone(&self) -> Box<dyn NFTsRepository + Send + Sync> {
        self.clone_box()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct ContractContentInfo {
    //field names coming from Solidity
    pub hashFile: String,
    pub hashAlgo: String,
    pub uri: Option<String>,
    pub price: Option<u64>,
    pub state: Option<ContentState>,
    pub token: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ContentState {
    Active,
    Inactive,
}

impl fmt::Debug for ContentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Inactive => write!(f, "Inactive"),
        }
    }
}

impl fmt::Display for ContentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Inactive => write!(f, "Inactive"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseContentStateError;
impl std::str::FromStr for ContentState {
    type Err = ParseContentStateError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "Active" => Ok(ContentState::Active),
            "Inactive" => Ok(ContentState::Inactive),
            _ => Err(ParseContentStateError),
        }
    }
}
