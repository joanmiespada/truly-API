use async_trait::async_trait;
use aws_sdk_dynamodb::{
    types::{AttributeValue, Put, Select, TransactWriteItem},
    Client,
};
use chrono::{
    prelude::{DateTime, Utc},
    Local,
};
use lib_config::config::Config;
use lib_licenses::errors::asset::AssetNoExistsError;
use std::{collections::HashMap, str::FromStr};
use uuid::Uuid;
//use web3::types::{H160, H256, U256, U64};

use crate::{
    errors::block_tx::{BlockchainTxError, BlockchainTxNoExistsError},
    models::block_tx::BlockchainTx,
};

use super::schema_block_tx::{
    TX_ASSET_ID_FIELD_PK, TX_FIELD, TX_INDEX_NAME, TX_TABLE_NAME, TX_TIMESTAMP_PK,
};
pub const TX_BLOCK_NUMER: &str = "block_numer";
pub const TX_GAS_USED: &str = "gas_used";
pub const TX_EFECTIVE_GAS_PRICE: &str = "effective_gas_price";
pub const TX_COST: &str = "cost";
pub const TX_CURRENCY: &str = "currency";
pub const TX_FROM: &str = "from";
pub const TX_TO: &str = "to";
pub const TX_CONTRACT_ID: &str = "contract_id";
pub const TX_ERROR: &str = "error";

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[async_trait]
pub trait BlockchainTxRepository {
    async fn add(&self, tx: &BlockchainTx) -> ResultE<()>;
    async fn get_by_tx(&self, hash: &String) -> ResultE<BlockchainTx>;
    async fn get_by_ids(&self, asset_id: &Uuid, timestamp: &DateTime<Utc>)
        -> ResultE<BlockchainTx>;
    async fn get_by_asset_id(&self, asset_id: &Uuid) -> ResultE<Vec<BlockchainTx>>;
}

#[derive(Clone, Debug)]
pub struct BlockchainTxRepo {
    client: Client,
}

impl BlockchainTxRepo {
    pub fn new(conf: &Config) -> BlockchainTxRepo {
        BlockchainTxRepo {
            client: Client::new(conf.aws_config()),
        }
    }
}

#[async_trait]
impl BlockchainTxRepository for BlockchainTxRepo {
    async fn get_by_tx(&self, hash: &String) -> ResultE<BlockchainTx> {
        let hash_av = AttributeValue::S(format!("{:?}", hash));

        let filter = format!("{} = :value", TX_FIELD);

        let request = self
            .client
            .query()
            .table_name(TX_TABLE_NAME)
            .index_name(TX_INDEX_NAME)
            .key_condition_expression(filter)
            .expression_attribute_values(":value".to_string(), hash_av)
            .select(Select::AllProjectedAttributes);

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(BlockchainTxError(e.to_string()).into());
            }
            Ok(items) => {
                if items.count() != 1 {
                    return Err(BlockchainTxError(format!(
                        "Tx hash is incorrect or duplicated. count:{} ",
                        items.count()
                    ))
                    .into());
                }
                let docus = items.items().unwrap();
                match docus.first() {
                    None => Err(BlockchainTxNoExistsError(hash.to_string()).into()),
                    Some(doc) => {
                        let _asset_id = doc.get(TX_ASSET_ID_FIELD_PK).unwrap();
                        let asset_id = _asset_id.as_s().unwrap();
                        let asset_uuid = Uuid::from_str(asset_id).unwrap();

                        let _time_id = doc.get(TX_TIMESTAMP_PK).unwrap();
                        let time_id = from_iso8601(_time_id.as_s().unwrap());

                        self.get_by_ids(&asset_uuid, &time_id).await
                    }
                }
            }
        }
    }

    async fn get_by_ids(
        &self,
        asset_id: &Uuid,
        timestamp: &DateTime<Utc>,
    ) -> ResultE<BlockchainTx> {
        let asset_id_av = AttributeValue::S(asset_id.to_string());
        let timestamp_av = AttributeValue::S(iso8601(timestamp));

        let request = self
            .client
            .get_item()
            .table_name(TX_TABLE_NAME)
            .key(TX_ASSET_ID_FIELD_PK, asset_id_av)
            .key(TX_TIMESTAMP_PK, timestamp_av);

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(BlockchainTxError(e.to_string()).into());
            }
            Ok(_) => {}
        }
        match results.unwrap().item {
            None => Err(BlockchainTxNoExistsError(format!(
                "ids doesn't exist asset: {} and timestamp: {}",
                asset_id.to_string(),
                iso8601(timestamp)
            ))
            .into()),
            Some(aux) => {
                let res = mapping_from_doc_to_blockchain(&aux);
                Ok(res)
            }
        }
    }

    async fn add(&self, tx: &BlockchainTx) -> ResultE<()> {
        let asset_id_av = AttributeValue::S(tx.asset_id().to_string());
        let creation_time_av = AttributeValue::S(iso8601(tx.creation_time()));
        let contract_id_av = AttributeValue::N(tx.contract_id().to_string());

        let mut items = Put::builder();
        items = items
            .item(TX_ASSET_ID_FIELD_PK, asset_id_av)
            .item(TX_TIMESTAMP_PK, creation_time_av)
            .item(TX_CONTRACT_ID, contract_id_av);

        match tx.tx() {
            None => {}
            Some(hash) => {
                let tx_id_av = AttributeValue::S(format!("{:?}", hash));
                items = items.item(TX_FIELD, tx_id_av);
            }
        }

        match tx.block_number() {
            None => {}
            Some(data) => {
                let data_av = AttributeValue::N(data.clone().to_string());
                items = items.item(TX_BLOCK_NUMER, data_av);
            }
        }

        match tx.gas_used() {
            None => {}
            Some(data) => {
                let data_av = AttributeValue::N(data.clone().to_string());
                items = items.item(TX_GAS_USED, data_av);
            }
        }

        match tx.effective_gas_price() {
            None => {}
            Some(data) => {
                let data_av = AttributeValue::N(data.clone().to_string());
                items = items.item(TX_EFECTIVE_GAS_PRICE, data_av);
            }
        }

        match tx.cost() {
            None => {}
            Some(data) => {
                let data_av = AttributeValue::N(data.clone().to_string());
                items = items.item(TX_COST, data_av);
            }
        }
        match tx.currency() {
            None => {}
            Some(data) => {
                let data_av = AttributeValue::S(data.clone().to_string());
                items = items.item(TX_CURRENCY, data_av);
            }
        }
        match tx.from() {
            None => {}
            Some(data) => {
                let data_av = AttributeValue::S(format!("{:?}", data));
                items = items.item(TX_FROM, data_av);
            }
        }
        match tx.to() {
            None => {}
            Some(data) => {
                let data_av = AttributeValue::S(format!("{:?}", data));
                items = items.item(TX_TO, data_av);
            }
        }
        match tx.tx_error() {
            None => {}
            Some(data) => {
                let data_av = AttributeValue::S(data.to_string());
                items = items.item(TX_ERROR, data_av);
            }
        }

        let request = self.client.transact_write_items().transact_items(
            TransactWriteItem::builder()
                .put(items.table_name(TX_TABLE_NAME).build())
                .build(),
        );

        match request.send().await {
            Ok(_) => Ok(()),
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(BlockchainTxError(e.to_string()).into());
            }
        }
    }

    async fn get_by_asset_id(&self, asset_id: &Uuid) -> ResultE<Vec<BlockchainTx>> {
        let mut queried = Vec::new();
        let asset_id_av = AttributeValue::S(asset_id.to_string());

        let mut filter = "".to_string();
        filter.push_str(TX_ASSET_ID_FIELD_PK);
        filter.push_str(" = :value");

        let request = self
            .client
            .query()
            .table_name(TX_TABLE_NAME)
            .key_condition_expression(filter)
            .expression_attribute_values(":value".to_string(), asset_id_av);
        //.select(Select::AllProjectedAttributes);
        //.key(OWNER_USER_ID_FIELD_PK, _id_av.clone());

        //let mut id_list = Vec::new();
        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(BlockchainTxError(e.to_string()).into());
            }
            Ok(data) => {
                let op_items = data.items();
                match op_items {
                    None => {
                        return Err(AssetNoExistsError("asset id doesn't exist".to_string()).into());
                    }
                    Some(aux) => {
                        for doc in aux {
                            let res = mapping_from_doc_to_blockchain(&doc);
                            queried.push(res.clone());
                        }
                    }
                }
            }
        }

        //for ass in assets_list {
        //let res = self._get_by_id(ass.asset_id()).await?;
        //let mut asset = Asset::new();
        //mapping_from_doc_to_asset(&res, &mut asset);

        //   let asset = self.get_by_id(ass.asset_id()).await?;

        //    queried.push(asset.clone());
        //}
        Ok(queried)
    }
}

fn iso8601(st: &DateTime<Utc>) -> String {
    let dt: DateTime<Utc> = st.clone().into();
    format!("{}", dt.format("%+"))
}

fn from_iso8601(st: &String) -> DateTime<Utc> {
    let aux = st.parse::<DateTime<Utc>>().unwrap();
    aux
}

pub fn mapping_from_doc_to_blockchain(doc: &HashMap<String, AttributeValue>) -> BlockchainTx {
    let _asset_id = doc.get(TX_ASSET_ID_FIELD_PK).unwrap();
    let asset_id1 = _asset_id.as_s().unwrap();
    let asset_id = Uuid::from_str(asset_id1).unwrap();

    let _creation_time = doc.get(TX_TIMESTAMP_PK).unwrap();
    let creation_time = from_iso8601(_creation_time.as_s().unwrap());

    let tx_hash;
    match doc.get(TX_FIELD) {
        None => tx_hash = None,
        Some(v) => {
            let tx_id = v.as_s().unwrap();
            //let hash = H256::from_str(tx_id).unwrap();
            tx_hash = Some(tx_id.clone());
        }
    }
    let block_numer;
    match doc.get(TX_BLOCK_NUMER) {
        None => block_numer = None,
        Some(v) => {
            let s_val = v.as_n().unwrap();
            //let val = U64::from_str(s_val).unwrap();
            let val = u64::from_str(s_val).unwrap();
            block_numer = Some(val);
        }
    }
    let gas_used;
    match doc.get(TX_GAS_USED) {
        None => gas_used = None,
        Some(v) => {
            let s_val = v.as_n().unwrap();
            //let val = U256::from_str(s_val).unwrap();
            gas_used = Some(s_val.clone());
        }
    }
    let effective_gas_price;
    match doc.get(TX_EFECTIVE_GAS_PRICE) {
        None => effective_gas_price = None,
        Some(v) => {
            let s_val = v.as_n().unwrap();
            //let val = U256::from_str(s_val).unwrap();
            effective_gas_price = Some(s_val.clone());
        }
    }
    let cost;
    match doc.get(TX_COST) {
        None => cost = None,
        Some(v) => {
            let s_val = v.as_n().unwrap();
            let val = f64::from_str(s_val).unwrap();
            cost = Some(val);
        }
    }
    let currency;
    match doc.get(TX_CURRENCY) {
        None => currency = None,
        Some(v) => {
            let val = v.as_s().unwrap().to_owned();
            currency = Some(val);
        }
    }
    let from;
    match doc.get(TX_FROM) {
        None => from = None,
        Some(v) => {
            let s_val = v.as_s().unwrap();
            //let val = H160::from_str(&s_val).unwrap();
            from = Some(s_val.clone());
        }
    }
    let to;
    match doc.get(TX_TO) {
        None => to = None,
        Some(v) => {
            let s_val = v.as_s().unwrap();
            //let val = H160::from_str(&s_val).unwrap();
            to = Some(s_val.clone());
        }
    }
    let _contract_id = doc.get(TX_CONTRACT_ID).unwrap();
    let contract_id1 = _contract_id.as_n().unwrap();
    let contract_id = u16::from_str(contract_id1).unwrap();

    let tx_error;
    match doc.get(TX_ERROR) {
        None => tx_error = None,
        Some(v) => {
            let s_val = v.as_s().unwrap().clone();
            tx_error = Some(s_val);
        }
    }

    let res = BlockchainTx::new(
        asset_id,
        creation_time,
        tx_hash,
        block_numer,
        gas_used,
        effective_gas_price,
        cost,
        currency,
        from,
        to,
        contract_id,
        tx_error,
    );
    res
}
