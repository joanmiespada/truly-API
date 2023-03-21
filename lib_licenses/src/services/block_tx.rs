

use std::str::FromStr;

use crate::models::tx::BlockchainTx ;
use crate::repositories::block_tx::{BlockchainTxRepository, BlockchainTxRepo};
use async_trait::async_trait;
use web3::types::H256;
use uuid::Uuid;
type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error +Sync + Send >>;

#[async_trait]
pub trait BlockchainTxManipulation {
    async fn add(&self, tx: & BlockchainTx) -> ResultE<()>;
    async fn get_by_hash(&self, hash: &H256) -> ResultE<BlockchainTx>;
    async fn get_by_id(&self, hash: &String) -> ResultE<BlockchainTx>;
    async fn get_by_asset_id(&self, asset_id: &Uuid) -> ResultE<Vec<BlockchainTx>>;
}

#[derive(Debug)]
pub struct BlockchainTxService {
    repository: BlockchainTxRepo,
}

impl BlockchainTxService {
    pub fn new(repo: BlockchainTxRepo) -> BlockchainTxService {
        BlockchainTxService { repository: repo }
    }
}


#[async_trait]
impl BlockchainTxManipulation for BlockchainTxService {

    #[tracing::instrument()]
    async fn add(&self, tx: &BlockchainTx) -> ResultE<()> {
        self.repository.add(tx).await
    }

    #[tracing::instrument()]
    async fn get_by_hash(&self, hash: &H256) -> ResultE<BlockchainTx>{
        self.repository.get_by_tx(hash).await
    }
    #[tracing::instrument()]
    async fn get_by_id(&self, hash: &String) -> ResultE<BlockchainTx>{
        let new_hash = H256::from_str(hash).unwrap();
        self.repository.get_by_tx(&new_hash).await
    }
    #[tracing::instrument()]
    async fn get_by_asset_id(&self, asset_id: &Uuid) -> ResultE<Vec<BlockchainTx>>{
        self.repository.get_by_asset_id(asset_id).await
    }
}

impl Clone for BlockchainTxService {
    #[tracing::instrument()]
    fn clone(&self) -> BlockchainTxService {
        let aux = BlockchainTxService {
            repository: self.repository.clone(),
        };
        return aux;
    }
}
