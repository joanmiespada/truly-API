use async_trait::async_trait;
use lib_licenses::models::asset::Asset;
use uuid::Uuid;

use crate::{models::Ledge, repository::{LedgerRepo, LedgerRepository}};
type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[async_trait]
pub trait LedgerManipulation {
    async fn add(&self, asset: &Asset) -> ResultE<Ledge>;
    async fn get_by_id(&self, tx: &String) -> ResultE<Ledge>;
    async fn get_by_asset_id(&self, asset_id: &Uuid) -> ResultE<Vec<Ledge>>;
}

#[derive(Debug)]
pub struct LedgerService {
    repository: LedgerRepo,
}

impl LedgerService {
    pub fn new(repo: LedgerRepo) -> LedgerService {
        LedgerService { repository: repo }
    }
}

#[async_trait]
impl LedgerManipulation for LedgerService {
    #[tracing::instrument()]
    async fn add(&self, asset: &Asset) -> ResultE<Ledge> {
        self.repository.add(asset).await
    }

    #[tracing::instrument()]
    async fn get_by_id(&self, tx: &String) -> ResultE<Ledge> {
        self.repository.get_by_id(tx).await
    }

    #[tracing::instrument()]
    async fn get_by_asset_id(&self, asset_id: &Uuid) -> ResultE<Vec<Ledge>> {
        self.repository.get_by_asset_id(asset_id).await
    }
}

impl Clone for LedgerService {
    #[tracing::instrument()]
    fn clone(&self) -> LedgerService {
        let aux = LedgerService {
            repository: self.repository.clone(),
        };
        return aux;
    }
}
