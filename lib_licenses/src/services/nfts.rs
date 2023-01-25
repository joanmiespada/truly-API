
use std::{fmt, str::FromStr};

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::repositories::ganache::{GanacheRepo, NFTsRepository, GanacheContentInfo};

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[async_trait]
pub trait NFTsManipulation {
    async fn add(&self, asset_id: &Uuid, user_address: &String, hash_file: &String, price: &u64 ) -> ResultE<String>;
    async fn get(&self, asset_id: &Uuid) -> ResultE<NTFContentInfo>;
}

#[derive(Debug)]
pub struct NFTsService(GanacheRepo);

impl NFTsService {
    pub fn new(repo: GanacheRepo  ) -> NFTsService {
        NFTsService(repo)
    }
}


#[async_trait]
impl NFTsManipulation for NFTsService {

    #[tracing::instrument()]
    async fn add(&self, asset_id: &Uuid, user_address: &String, hash_file: &String, price: &u64 ) -> ResultE<String> {
        let aux = self.0.add(asset_id, user_address, hash_file, price).await?;
        Ok(aux)
    }
    
    #[tracing::instrument()]
    async fn get(&self, asset_id: &Uuid) -> ResultE<NTFContentInfo> {
        let aux =self.0.get(asset_id).await?;
        let res = NTFContentInfo{
            hashFile: aux.hashFile,
            uri: aux.uri,
            price: aux.price,
            state : NTFState::from_str( &aux.state.to_string()).unwrap()
        };
        Ok(res)
    }
    
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NTFContentInfo {
    pub hashFile: String,
    pub uri: String,
    pub price: u64,
    pub state: NTFState,
}

#[derive(Clone, Serialize, Deserialize,PartialEq, Eq)]
pub enum NTFState {
    Active,
    Inactive
}
impl fmt::Debug for NTFState{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Inactive => write!(f, "Inactive"),
        }
    }
}
impl fmt::Display for NTFState{
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