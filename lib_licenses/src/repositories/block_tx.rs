use async_trait::async_trait;
use aws_sdk_dynamodb::{
    model::{AttributeValue, Put, Select, TransactWriteItem},
    Client,
};
use chrono::{
    prelude::{DateTime, Utc},
    Local,
};
use lib_config::config::Config;
use std::{collections::HashMap, str::FromStr};
use uuid::Uuid;
use web3::types::H256;

use crate::{
    errors::block_tx::{BlockchainTxError, BlockchainTxNoExistsError},
    models::tx::BlockchainTx,
};

use super::{
    schema_block_tx::{
        BLOCKCHAIN_TX_TABLE_NAME, TX_ASSET_ID_FIELD_PK, TX_FIELD, TX_INDEX_NAME, TX_TIMESTAMP_PK,
    },
};
pub const PAYLOAD_FIELD_NAME: &str = "details"; //TODO: split in multiples columns

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[async_trait]
pub trait BlockchainTxRepository {
    async fn add(&self, tx: &BlockchainTx) -> ResultE<()>;
    async fn get_by_tx(&self, hash: &H256) -> ResultE<BlockchainTx>;
    async fn get_by_ids(&self, asset_id: &Uuid, timestamp: &DateTime<Utc>)
        -> ResultE<BlockchainTx>;
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
    async fn get_by_tx(&self, hash: &H256) -> ResultE<BlockchainTx> {
        let hash_av = AttributeValue::S(format!("{:?}", hash));

        let filter = format!("{} = :value", TX_FIELD);

        let request = self
            .client
            .query()
            .table_name(BLOCKCHAIN_TX_TABLE_NAME)
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
            .table_name(BLOCKCHAIN_TX_TABLE_NAME)
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
                let mut res = BlockchainTx::new();
                mapping_from_doc_to_blockchain(&aux, &mut res);
                Ok(res)
            }
        }
    }

    async fn add(&self, tx: &BlockchainTx) -> ResultE<()> {
        let asset_id_av = AttributeValue::S(tx.asset_id().to_string());
        let creation_time_av = AttributeValue::S(iso8601(tx.creation_time()));

        let mut items = Put::builder();
        items = items
            .item(TX_ASSET_ID_FIELD_PK, asset_id_av)
            .item(TX_TIMESTAMP_PK, creation_time_av);
        match tx.tx() {
            None => {}
            Some(hash) => {
                let tx_id_av = AttributeValue::S(format!("{:?}", hash));
                items = items.item(TX_FIELD, tx_id_av);
            }
        }

        match tx.result() {
            None => {}
            Some(data) => {
                let data_av = AttributeValue::S(data.clone());
                items = items.item(PAYLOAD_FIELD_NAME, data_av);
            }
        }
        let request = self.client.transact_write_items().transact_items(
            TransactWriteItem::builder()
                .put(items.table_name(BLOCKCHAIN_TX_TABLE_NAME).build())
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
}

fn iso8601(st: &DateTime<Utc>) -> String {
    let dt: DateTime<Utc> = st.clone().into();
    format!("{}", dt.format("%+"))
}

fn from_iso8601(st: &String) -> DateTime<Utc> {
    let aux = st.parse::<DateTime<Utc>>().unwrap();
    aux
}

pub fn mapping_from_doc_to_blockchain(
    doc: &HashMap<String, AttributeValue>,
    tx: &mut BlockchainTx,
) {
    let _asset_id = doc.get(TX_ASSET_ID_FIELD_PK).unwrap();
    let asset_id = _asset_id.as_s().unwrap();
    let uuid = Uuid::from_str(asset_id).unwrap();
    tx.set_asset_id(&uuid);

    match doc.get(TX_FIELD) {
        None => {}
        Some(v) => {
            let tx_id = v.as_s().unwrap();
            let hash = H256::from_str(tx_id).unwrap();
            tx.set_tx(&hash);
        }
    }

    match doc.get(TX_TIMESTAMP_PK) {
        None => {}
        Some(creation_time) => {
            tx.set_creation_time(&from_iso8601(creation_time.as_s().unwrap()));
        }
    }

    match doc.get(PAYLOAD_FIELD_NAME) {
        None => {}
        Some(info) => {
            tx.set_result(&info.as_s().unwrap());
        }
    }
    
}
