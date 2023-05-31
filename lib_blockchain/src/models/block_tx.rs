use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
use uuid::Uuid;
use validator::Validate;
//use web3::types::{H160, H256, U256, U64};

#[derive(Clone, Serialize, Validate, Deserialize, Debug)]
pub struct BlockchainTx {
    asset_id: Uuid,
    creation_time: DateTime<Utc>,
    tx_hash: Option<String>,
    block_number: Option<u64>,
    gas_used: Option<String>,
    effective_gas_price: Option<String>,
    cost: Option<f64>,
    currency: Option<String>,
    from: Option<String>,
    to: Option<String>,
    contract_id: u16,
    tx_error: Option<String>,
}

impl fmt::Display for BlockchainTx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", json!(self).to_string())
    }
}

impl BlockchainTx {
    pub fn new(
        asset_id: Uuid,
        creation_time: DateTime<Utc>,
        tx_hash: Option<String>,
        block_number: Option<u64>,
        gas_used: Option<String>,
        effective_gas_price: Option<String>,
        cost: Option<f64>,
        currency: Option<String>,
        from: Option<String>,
        to: Option<String>,
        contract_id: u16,
        tx_error: Option<String>,
    ) -> BlockchainTx {
        //let creation_time = Utc::now();
        BlockchainTx {
            asset_id,
            creation_time,
            tx_hash,
            block_number,
            gas_used,
            effective_gas_price,
            cost,
            currency,
            from,
            to,
            contract_id,
            tx_error,
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

    pub fn tx(&self) -> &Option<String> {
        &self.tx_hash
    }
    pub fn set_tx(&mut self, val: &String) {
        self.tx_hash = Some(val.clone())
    }
    pub fn block_number(&self) -> &Option<u64> {
        &self.block_number
    }
    pub fn set_block_number(&mut self, val: &u64) {
        self.block_number = Some(val.clone())
    }
    pub fn gas_used(&self) -> &Option<String> {
        &self.gas_used
    }
    pub fn set_gas_used(&mut self, val: &String) {
        self.gas_used = Some(val.clone())
    }

    pub fn effective_gas_price(&self) -> &Option<String> {
        &self.effective_gas_price
    }
    pub fn set_effective_gas_price(&mut self, val: &String) {
        self.effective_gas_price = Some(val.clone())
    }
    pub fn cost(&self) -> &Option<f64> {
        &self.cost
    }
    pub fn set_cost(&mut self, val: &f64) {
        self.cost = Some(val.clone())
    }
    pub fn currency(&self) -> &Option<String> {
        &self.currency
    }
    pub fn set_currency(&mut self, val: &String) {
        self.currency = Some(val.clone())
    }
    pub fn from(&self) -> &Option<String> {
        &self.from
    }
    pub fn set_from(&mut self, val: &String) {
        self.from = Some(val.clone())
    }
    pub fn to(&self) -> &Option<String> {
        &self.to
    }
    pub fn set_to(&mut self, val: &String) {
        self.to = Some(val.clone())
    }
    pub fn contract_id(&self) -> &u16 {
        &self.contract_id
    }
    pub fn set_contract_id(&mut self, val: &u16) {
        self.contract_id = val.clone();
    }
    pub fn tx_error(&self) -> &Option<String> {
        &self.tx_error
    }
    pub fn set_tx_error(&mut self, val: &String) {
        self.tx_error = Some(val.clone())
    }
}
