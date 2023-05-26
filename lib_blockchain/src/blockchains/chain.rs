use std::fmt;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::block_tx::BlockchainTx,
};
use crate::{
    models::keypair::KeyPair,
};

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[async_trait]
pub trait NFTsRepository {
    async fn add(
        &self,
        asset_id: &Uuid,
        user_key: &KeyPair,
        hash_file: &String,
        price: &u64,
        counter: &u64,
    ) -> ResultE<BlockchainTx>;

    async fn get(&self, asset_id: &Uuid) -> ResultE<ContractContentInfo>;
    fn contract_id(&self) -> u16;
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct ContractContentInfo {
    //field names coming from Solidity
    pub hashFile: String,
    pub uri: String,
    pub price: u64,
    pub state: ContentState,
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

