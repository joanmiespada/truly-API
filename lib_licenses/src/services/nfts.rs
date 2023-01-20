
use async_trait::async_trait;
use uuid::Uuid;

use crate::repositories::ganache::{GanacheRepo, NFTsRepository};

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[async_trait]
pub trait NFTsManipulation {
    async fn add(&self, asset_id: &Uuid, user_id: &String) -> ResultE<()>;
}

#[derive(Debug)]
pub struct NFTsService {
    blockchain: GanacheRepo
}

impl NFTsService {
    pub fn new(blockchain: GanacheRepo ) -> NFTsService {
        NFTsService { blockchain:blockchain  }
    }
}


#[async_trait]
impl NFTsManipulation for NFTsService {
    

    #[tracing::instrument()]
    async fn add(&self, asset_id: &Uuid, user_id: &String) -> ResultE<()> {
        self.blockchain.add(asset_id, user_id)?;
        Ok(())
    }
    
}

