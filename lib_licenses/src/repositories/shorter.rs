use std::str::FromStr;

use async_trait::async_trait;
use aws_sdk_dynamodb::{
    types::{AttributeValue, Put, TransactWriteItem},
    Client,
};
use chrono::Local;
use lib_config::config::Config;
use uuid::Uuid;

use crate::errors::asset::{AssetDynamoDBError, AssetNoExistsError};

use super::schema_asset::{SHORTER_ASSET_ID_FIELD, SHORTER_FIELD_PK, SHORTER_TABLE_NAME};

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[async_trait]
pub trait ShorterRepository {
    async fn add(&self, asset_id: &Uuid, shorter_id: &String) -> ResultE<()>;
    async fn get_by_shorter(&self, shorter_id: &String) -> ResultE<Uuid>;
}

#[derive(Clone, Debug)]
pub struct ShorterRepo {
    client: Client,
}

impl ShorterRepo {
    pub fn new(conf: &Config) -> ShorterRepo {
        ShorterRepo {
            client: Client::new(conf.aws_config()),
        }
    }
}

#[async_trait]
impl ShorterRepository for ShorterRepo {
    async fn get_by_shorter(&self, shorter_id: &String) -> ResultE<Uuid> {
        let shorter_av = AttributeValue::S(shorter_id.to_owned());

        let mut filter = "".to_string();
        filter.push_str(SHORTER_FIELD_PK);
        filter.push_str(" = :value");

        let request = self
            .client
            .get_item()
            .table_name(SHORTER_TABLE_NAME.clone())
            .key(SHORTER_FIELD_PK, shorter_av);

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                log::error!("{}",mssag);
                return Err(AssetDynamoDBError(e.to_string()).into());
            }
            Ok(_) => {}
        }
        match results.unwrap().item {
            None => Err(AssetNoExistsError("shorter doesn't exist".to_string()).into()),
            Some(aux) => {
                let ass1_id = aux.get(SHORTER_ASSET_ID_FIELD).unwrap();
                let ass1_id1 = ass1_id.as_s().unwrap();
                let ass1_id1_1 = Uuid::from_str(ass1_id1).unwrap();
                Ok(ass1_id1_1)
            }
        }
    }

    async fn add(&self, asset_id: &Uuid, shorter_id: &String) -> ResultE<()> {
        let asset_id_av = AttributeValue::S(asset_id.to_string());
        let shorter_id_av = AttributeValue::S(shorter_id.to_owned());

        let mut items = Put::builder();
        items = items
            .item(SHORTER_ASSET_ID_FIELD, asset_id_av)
            .item(SHORTER_FIELD_PK, shorter_id_av);

        let request = self.client.transact_write_items().transact_items(
            TransactWriteItem::builder()
                .put(items.table_name(SHORTER_TABLE_NAME.clone()).build())
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
                log::error!("{}",mssag);
                return Err(AssetDynamoDBError(e.to_string()).into());
            }
        }
    }
}
