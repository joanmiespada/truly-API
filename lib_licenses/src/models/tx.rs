use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
use uuid::Uuid;
use validator::Validate;
use web3::types::H256;


#[derive(Clone, Serialize,Validate, Deserialize, Debug)]
pub struct BlockchainTx {
    asset_id: Uuid,
    creation_time: DateTime<Utc>,
    result: Option<String>,
    tx_hash: Option<H256>
}

impl fmt::Display for BlockchainTx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f,"{}", json!(self).to_string())
    }
}

impl BlockchainTx {
    pub fn new() -> BlockchainTx {
        BlockchainTx {
            asset_id: Uuid::nil(),
            creation_time: Utc::now(),
            result: None,
            tx_hash: None
        }
    }

    pub fn asset_id(&self) -> &Uuid {
        &self.asset_id
    }
    pub fn set_asset_id(&mut self, val: &Uuid) {
        self.asset_id = val.clone()
    }
    pub fn creation_time(&self) -> &DateTime<Utc> {
        &self.creation_time
    }
    pub fn set_creation_time(&mut self, val: &DateTime<Utc>) {
        self.creation_time = val.clone()
    }
    pub fn result(&self) -> &Option<String> {
        &self.result
    }
    pub fn set_result(&mut self, val: &String) {
        self.result = Some(val.clone())
    }
    
    pub fn tx(&self) -> &Option<H256> {
        &self.tx_hash
    }
    pub fn set_tx(&mut self, val: &H256) {
        self.tx_hash = Some(val.clone())
    }
}