

use crate::models::tx::BlockchainTx ;
use crate::repositories::block_tx::{BlockchainTxRepository, BlockchainTxRepo};
use async_trait::async_trait;
use web3::types::H256;

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error +Sync + Send >>;

#[async_trait]
pub trait BlockchainTxManipulation {
    async fn add(&self, tx: & BlockchainTx) -> ResultE<()>;
    async fn get_by_tx(&self, hash: &H256) -> ResultE<BlockchainTx>;
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
    async fn get_by_tx(&self, hash: &H256) -> ResultE<BlockchainTx>{
        self.repository.get_by_tx(hash).await
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
