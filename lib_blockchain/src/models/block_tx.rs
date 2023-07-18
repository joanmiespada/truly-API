use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fmt, str::FromStr};
use uuid::Uuid;
use validator::Validate;
//use web3::types::{H160, H256, U256, U64};

#[derive(Clone, Serialize, Validate, Deserialize, Debug, PartialEq)]
pub struct BlockchainTx {
    asset_id: Uuid,
    mint_status: MintingStatus,
    creation_time: DateTime<Utc>,
    last_update_time: DateTime<Utc>,
    tx_hash: Option<String>,
    block_number: Option<u64>,
    gas_used: Option<String>,
    effective_gas_price: Option<String>,
    cost: Option<f64>,
    currency: Option<String>,
    from: Option<String>,
    to: Option<String>,
    contract_id: Option<u16>,
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
        mint_status: MintingStatus,
        creation_time: DateTime<Utc>,
        last_update_time: DateTime<Utc>,
        tx_hash: Option<String>,
        block_number: Option<u64>,
        gas_used: Option<String>,
        effective_gas_price: Option<String>,
        cost: Option<f64>,
        currency: Option<String>,
        from: Option<String>,
        to: Option<String>,
        contract_id: Option<u16>,
        tx_error: Option<String>,
    ) -> BlockchainTx {
        BlockchainTx {
            asset_id,
            mint_status,
            creation_time,
            last_update_time,
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
    pub fn last_update_time(&self) -> &DateTime<Utc> {
        &self.last_update_time
    }
    pub fn set_last_update_time(&mut self, val: &DateTime<Utc>) {
        self.last_update_time = val.clone()
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
    pub fn contract_id(&self) -> &Option<u16> {
        &self.contract_id
    }
    pub fn set_contract_id(&mut self, val: &u16) {
        self.contract_id = Some(val.clone());
    }
    pub fn tx_error(&self) -> &Option<String> {
        &self.tx_error
    }
    pub fn set_tx_error(&mut self, val: &String) {
        self.tx_error = Some(val.clone())
    }
    pub fn mint_status(&self) -> MintingStatus {
        self.mint_status.clone()
    }
    pub fn set_minted_status(&mut self, val: MintingStatus) {
        self.mint_status = val.clone()
    }
}

impl Default for BlockchainTx {
    fn default() -> Self {
        Self {
            asset_id: Default::default(),
            mint_status: MintingStatus::NeverMinted,
            creation_time: Utc::now(),
            last_update_time: Utc::now(),
            tx_hash: Default::default(),
            block_number: Default::default(),
            gas_used: Default::default(),
            effective_gas_price: Default::default(),
            cost: Default::default(),
            currency: Default::default(),
            from: Default::default(),
            to: Default::default(),
            contract_id: Default::default(),
            tx_error: Default::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum MintingStatus {
    NeverMinted,
    Scheduled,
    Started,
    CompletedSuccessfully,
    Error,
}

impl fmt::Display for MintingStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MintingStatus::Scheduled => write!(f, "Scheduled"),
            MintingStatus::Started => write!(f, "Started"),
            MintingStatus::CompletedSuccessfully => write!(f, "Completed successfully"),
            MintingStatus::Error => write!(f, "Error"),
            MintingStatus::NeverMinted => write!(f, "Never minted"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MintinStatusParseError;

impl FromStr for MintingStatus {
    type Err = MintinStatusParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Scheduled" => Ok(MintingStatus::Scheduled),
            "Started" => Ok(MintingStatus::Started),
            "Completed successfully" => Ok(MintingStatus::CompletedSuccessfully),
            "Error" => Ok(MintingStatus::Error),
            "Never minted" => Ok(MintingStatus::NeverMinted),
            _ => Err(MintinStatusParseError),
        }
    }
}

pub struct BlockchainTxBuilder {
    asset_id: Uuid,
    mint_status: MintingStatus
}

impl BlockchainTxBuilder {
    pub fn new() -> BlockchainTxBuilder {
        BlockchainTxBuilder{
            asset_id: Uuid::default(),
            mint_status: MintingStatus::NeverMinted
        }
    }
    pub fn asset_id(&mut self, id: Uuid) -> &mut BlockchainTxBuilder {
        self.asset_id = id.clone();
        self
    }
    pub fn mint_status(&mut self, state: MintingStatus) -> &mut BlockchainTxBuilder {
        self.mint_status = state;
        self
    }
    

    pub fn build(&self) -> BlockchainTx {
        let mut res = BlockchainTx::default();
        res.set_asset_id(&self.asset_id);
        res.set_minted_status(self.mint_status.clone());

        res
    }
}