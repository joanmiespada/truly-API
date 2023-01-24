
use async_trait::async_trait;
use uuid::Uuid;

use crate::repositories::ganache::{GanacheRepo, NFTsRepository};

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[async_trait]
pub trait NFTsManipulation {
    async fn add(&self, asset_id: &Uuid, user_address: &String, hash_file: &String, price: &u64 ) -> ResultE<()>;
    async fn get(&self, asset_id: &Uuid) -> ResultE<String>;
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
    async fn add(&self, asset_id: &Uuid, user_address: &String, hash_file: &String, price: &u64 ) -> ResultE<()> {
        self.0.add(asset_id, user_address, hash_file, price).await?;
        Ok(())
    }
    
    #[tracing::instrument()]
    async fn get(&self, asset_id: &Uuid) -> ResultE<String> {
        let res =self.0.get(asset_id).await?;
        Ok(res)
    }
    
}

