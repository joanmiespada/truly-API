use std::collections::HashMap;

use chrono::{DateTime, Utc};
use lib_config::timing::iso8601;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::repository::schema_ledger::{LEDGER_FIELD_HASH, LEDGER_FIELD_ASSET_ID, 
        LEDGER_FIELD_HASH_ALGO, LEDGER_FIELD_CREATION_TIME};


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Ledge {
    
    pub blockchain_id: String, 
    pub blockchain_seq: u64,
    pub hash: String,

    pub metadata_id: String,
    pub metadata_tx_time: DateTime<Utc>,
    pub metadata_tx_id: String,
    pub metadata_version: u64,

}

impl Default for Ledge {
    fn default() -> Self {
        Self {
            blockchain_id: Default::default(), 
            blockchain_seq: Default::default(),
            hash: Default::default(),
            metadata_id:Default::default(),
            metadata_tx_time:Default::default(),
            metadata_tx_id:Default::default(),
            metadata_version:Default::default()
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AssetLedged {
    pub asset_id: Uuid,
    pub asset_hash: String,
    pub asset_hash_algorithm: String,
    pub asset_creation_time: DateTime<Utc>,
}

impl AssetLedged {
    pub fn to_hash_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert( LEDGER_FIELD_ASSET_ID.to_string(), self.asset_id.to_string());
        map.insert( LEDGER_FIELD_HASH.to_string()  , self.asset_hash.clone());
        map.insert( LEDGER_FIELD_HASH_ALGO.to_string(), self.asset_hash_algorithm.clone());
        map.insert( LEDGER_FIELD_CREATION_TIME.to_string(), iso8601(&self.asset_creation_time));
        map
    }
}



