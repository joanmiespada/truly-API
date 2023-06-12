use std::collections::HashMap;

use crate::errors::keypair::KeyPairDynamoDBError;
use crate::models::keypair::KeyPair;
use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{
    prelude::{DateTime, Utc},
    Local,
};
use lib_config::config::Config;
//use rand::{prelude::*, SeedableRng};

use super::schema_keypairs::{
    KEYPAIRS_ADDRESS_FIELD, KEYPAIRS_PRIVATE_FIELD, KEYPAIRS_PUBLIC_FIELD, KEYPAIRS_TABLE_NAME,
    KEYPAIRS_USER_ID_FIELD_PK,
};
pub const CREATIONTIME_FIELD_NAME: &str = "creationTime";
pub const LASTUPDATETIME_FIELD_NAME: &str = "lastUpdateTime";

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[async_trait]
pub trait KeyPairRepository {
    async fn add(&self, keypair: &KeyPair) -> ResultE<()>;
    async fn get_by_id(&self, user_id: &String) -> ResultE<Option<KeyPair>>;
    //async fn save(&self, user_id: &String, keypair: &KeyPair) -> ResultE<()>;
}

#[derive(Clone, Debug)]
pub struct KeyPairRepo {
    client_dynamo: aws_sdk_dynamodb::Client,
    //client_kms: aws_sdk_kms::Client,
    //kms_key_id: String,
}

impl KeyPairRepo {
    pub fn new(conf: &Config) -> KeyPairRepo {
        KeyPairRepo {
            client_dynamo: aws_sdk_dynamodb::Client::new(conf.aws_config()),
            //client_kms: aws_sdk_kms::Client::new(conf.aws_config()),
            //kms_key_id: conf.env_vars().kms_key_id().to_owned(),
        }
    }
}

#[async_trait]
impl KeyPairRepository for KeyPairRepo {
    async fn add(&self, keypair: &KeyPair) -> ResultE<()> {
        let user_id_av = AttributeValue::S(keypair.user_id().to_string());
        let address_av = AttributeValue::S(keypair.address().to_string());
        let public_key_av = AttributeValue::S(keypair.public_key().to_string());
        let private_key_av = AttributeValue::S(keypair.private_key().to_string());
        let creation_time_av = AttributeValue::S(iso8601(keypair.creation_time()));
        let update_time_av = AttributeValue::S(iso8601(keypair.creation_time()));

        let request = self
            .client_dynamo
            .put_item()
            .table_name(KEYPAIRS_TABLE_NAME)
            .item(KEYPAIRS_USER_ID_FIELD_PK, user_id_av)
            .item(KEYPAIRS_ADDRESS_FIELD, address_av)
            .item(KEYPAIRS_PRIVATE_FIELD, private_key_av)
            .item(KEYPAIRS_PUBLIC_FIELD, public_key_av)
            .item(CREATIONTIME_FIELD_NAME, creation_time_av)
            .item(LASTUPDATETIME_FIELD_NAME, update_time_av);

        match request.send().await {
            Ok(_) => Ok(()),
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(KeyPairDynamoDBError(e.to_string()).into());
            }
        }
    }

    async fn get_by_id(&self, user_id: &String) -> ResultE<Option<KeyPair>> {
        let _id_av = AttributeValue::S(user_id.to_string());
        let request = self
            .client_dynamo
            .get_item()
            .table_name(KEYPAIRS_TABLE_NAME)
            .key(KEYPAIRS_USER_ID_FIELD_PK, _id_av.clone());

        let results = request.send().await;
        if let Err(e) = results {
            let mssag = format!(
                "Error at [{}] - {} ",
                Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                e
            );
            tracing::error!(mssag);
            return Err(KeyPairDynamoDBError(e.to_string()).into());
        }
        match results.unwrap().item {
            None => Ok(None), //Err(KeyPairNoExistsError("id doesn't exist".to_string()).into()),
            Some(aux) => {
                let mut keypair = KeyPair::new();

                mapping_from_doc_to_keypair(&aux, &mut keypair);

                Ok(Some(keypair))
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

pub fn mapping_from_doc_to_keypair(doc: &HashMap<String, AttributeValue>, keypair: &mut KeyPair) {
    let user_id = doc.get(KEYPAIRS_USER_ID_FIELD_PK).unwrap();
    let user_id = user_id.as_s().unwrap();
    //let uuid = Uuid::from_str(keypair_id).unwrap();
    keypair.set_user_id(&user_id);

    let _address = doc.get(KEYPAIRS_ADDRESS_FIELD).unwrap();
    let address = _address.as_s().unwrap();
    keypair.set_address(address);

    let _public_key = doc.get(KEYPAIRS_PUBLIC_FIELD).unwrap();
    let public_key = _public_key.as_s().unwrap();
    keypair.set_public_key(public_key);

    let _private_key = doc.get(KEYPAIRS_PRIVATE_FIELD).unwrap();
    let private_key = _private_key.as_s().unwrap();
    keypair.set_private_key(private_key);

    let creation_time_t = doc.get(CREATIONTIME_FIELD_NAME);
    match creation_time_t {
        None => {}
        Some(creation_time) => {
            keypair.set_creation_time(&from_iso8601(creation_time.as_s().unwrap()));
        }
    }

    let last_update_time_t = doc.get(LASTUPDATETIME_FIELD_NAME);
    match last_update_time_t {
        None => {}
        Some(last_update_time) => {
            keypair.set_last_update_time(&from_iso8601(last_update_time.as_s().unwrap()));
        }
    }
}
