use std::{collections::HashMap, str::FromStr};

use async_trait::async_trait;
use aws_sdk_dynamodb::model::{AttributeValue, Put, TransactWriteItem};
use chrono::Local;
use lib_config::config::Config;
use url::Url;

use crate::{
    errors::blockchain::{BlockchainDynamoDBError, BlockchainNoExistsError},
    models::blockchain::Blockchain,
};

use super::schema_blockchain::{BLOCKCHAIN_ID_FIELD_PK, BLOCKCHAIN_TABLE_NAME};

pub const BLOCKCHAIN_URL_FIELD_NAME: &str = "url";
pub const BLOCKCHAIN_API_KEY_FIELD_NAME: &str = "api_key";
pub const BLOCKCHAIN_CONFIRMATIONS_FIELD_NAME: &str = "confirmations";
pub const BLOCKCHAIN_EXPLORER_URL_FIELD_NAME: &str = "explorer";
pub const BLOCKCHAIN_EXPLORER_API_KEY_FIELD_NAME: &str = "explorer_api_key";

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[async_trait]
pub trait BlockchainRepository {
    async fn add(&self, cont: &Blockchain) -> ResultE<()>;
    async fn update(&self, cont: &Blockchain) -> ResultE<()>;
    async fn get_by_id(&self, id: &String) -> ResultE<Blockchain>;
}

#[derive(Clone, Debug)]
pub struct BlockchainRepo {
    client: aws_sdk_dynamodb::Client,
}

impl BlockchainRepo {
    pub fn new(conf: &Config) -> BlockchainRepo {
        BlockchainRepo {
            client: aws_sdk_dynamodb::Client::new(conf.aws_config()),
        }
    }

    fn new_or_update(
        &self,
        blockchain: &Blockchain,
    ) -> ResultE<aws_sdk_dynamodb::model::put::Builder> {
        let id_av = AttributeValue::S(blockchain.id().to_string());

        let url_av = AttributeValue::S(blockchain.url().to_string());
        let api_key_av = AttributeValue::S(blockchain.api_key().to_owned());
        let confirmations_av = AttributeValue::N(blockchain.confirmations().to_string());
        let explorer_av = AttributeValue::S(blockchain.explorer().to_string());
        let explorer_api_key_av = AttributeValue::S(blockchain.explorer_api_key().to_owned());

        let mut items = Put::builder();
        items = items
            .item(BLOCKCHAIN_ID_FIELD_PK, id_av)
            .item(BLOCKCHAIN_URL_FIELD_NAME, url_av)
            .item(BLOCKCHAIN_API_KEY_FIELD_NAME, api_key_av)
            .item(BLOCKCHAIN_CONFIRMATIONS_FIELD_NAME, confirmations_av)
            .item(BLOCKCHAIN_EXPLORER_URL_FIELD_NAME, explorer_av)
            .item(BLOCKCHAIN_EXPLORER_API_KEY_FIELD_NAME, explorer_api_key_av);
        Ok(items)
    }
}

#[async_trait]
impl BlockchainRepository for BlockchainRepo {
    async fn add(&self, blockchain: &Blockchain) -> ResultE<()> {
        let items = self.new_or_update(blockchain).unwrap();

        let request = self.client.transact_write_items().transact_items(
            TransactWriteItem::builder()
                .put(items.table_name(BLOCKCHAIN_TABLE_NAME).build())
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
                return Err(BlockchainDynamoDBError(e.to_string()).into());
            }
        }
    }
    async fn update(&self, blockchain: &Blockchain) -> ResultE<()> {
        let items = self.new_or_update(blockchain).unwrap();

        let request = self.client.transact_write_items().transact_items(
            TransactWriteItem::builder()
                .put(items.table_name(BLOCKCHAIN_TABLE_NAME).build())
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
                return Err(BlockchainDynamoDBError(e.to_string()).into());
            }
        }
    }
    async fn get_by_id(&self, id: &String) -> ResultE<Blockchain> {
        let id_av = AttributeValue::S(id.to_string());
        let request = self
            .client
            .get_item()
            .table_name(BLOCKCHAIN_TABLE_NAME)
            .key(BLOCKCHAIN_ID_FIELD_PK, id_av.clone());

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(BlockchainDynamoDBError(e.to_string()).into());
            }
            Ok(_) => {}
        }
        match results.unwrap().item {
            None => Err(BlockchainNoExistsError("id doesn't exist".to_string()).into()),
            Some(aux) => {

                let cont = mapping_from_doc_to_blockchain(&aux);

                Ok(cont)
            }
        }
    }
}

pub fn mapping_from_doc_to_blockchain(
    doc: &HashMap<String, AttributeValue>) ->Blockchain
{ 
    
    let _id = doc.get(BLOCKCHAIN_ID_FIELD_PK).unwrap();
    let id = _id.as_s().unwrap().to_owned();

    let _url = doc.get(BLOCKCHAIN_URL_FIELD_NAME ).unwrap();
    let url1 = _url.as_s().unwrap();
    let url = Url::from_str(url1.as_str()).unwrap();

    let _api_key= doc.get( BLOCKCHAIN_API_KEY_FIELD_NAME ).unwrap();
    let api_key= _api_key.as_s().unwrap().to_owned();

    let _confir = doc.get( BLOCKCHAIN_CONFIRMATIONS_FIELD_NAME).unwrap();
    let confirm = _confir.as_n().unwrap();
    let confirmations = u16::from_str(confirm).unwrap();

    let _explorer = doc.get(BLOCKCHAIN_EXPLORER_URL_FIELD_NAME).unwrap();
    let explorer1 = _explorer.as_s().unwrap();
    let explorer = Url::from_str(explorer1.as_str()).unwrap();

    let _explorer_api_key = doc.get(BLOCKCHAIN_EXPLORER_API_KEY_FIELD_NAME).unwrap();
    let explorer_api_key = _explorer_api_key.as_s().unwrap().to_owned();

    let res = Blockchain::new(
        id,
        url,
        api_key,
        confirmations,
        explorer,
        explorer_api_key
    );
    return res;

}
