use std::collections::HashMap;

use crate::errors::keypair::{KeyPairDynamoDBError, KeyPairNoExistsError};
use crate::models::keypair::KeyPair;
use async_trait::async_trait;
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_kms::types::Blob;
use chrono::{
    prelude::{DateTime, Utc},
    Local,
};
use lib_config::config::Config;
use secp256k1::rand::{rngs, SeedableRng};
use web3::signing::keccak256;
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
    async fn get_by_id(&self, user_id: &String) -> ResultE<KeyPair>;
    async fn get_or_create(&self, user_id: &String) -> ResultE<KeyPair>;
}

#[derive(Clone, Debug)]
pub struct KeyPairRepo {
    client_dynamo: aws_sdk_dynamodb::Client,
    client_kms: aws_sdk_kms::Client,
    kms_key_id: String,
}

impl KeyPairRepo {
    pub fn new(conf: &Config) -> KeyPairRepo {
        KeyPairRepo {
            client_dynamo: aws_sdk_dynamodb::Client::new(conf.aws_config()),
            client_kms: aws_sdk_kms::Client::new(conf.aws_config()),
            kms_key_id: conf.env_vars().kms_key_id().to_owned(),
        }
    }
    async fn encrypt(&self, info_to_be_encrypted: String) -> ResultE<String> {
        use base64::{engine::general_purpose, Engine as _};
        let blob = Blob::new(info_to_be_encrypted.as_bytes());
        let resp;
        let resp_op = self
            .client_kms
            .encrypt()
            .key_id(self.kms_key_id.clone())
            .plaintext(blob)
            .send()
            .await;
        match resp_op{
            Err(e)=>{
                panic!("{}",e)
            },
            Ok(value)=>{resp=value}
        }

        let blob = resp.ciphertext_blob.unwrap();
        let bytes = blob.as_ref();

        let value = general_purpose::STANDARD.encode(bytes);

        Ok(value)
    }
    async fn _decrypt(&self, info_encrypted_b64: String) -> ResultE<String>{
        use base64::{engine::general_purpose, Engine as _};

        let value = general_purpose::STANDARD.decode( info_encrypted_b64 ).unwrap();

        let data = aws_sdk_kms::types::Blob::new(value);

        let resp = self
            .client_kms
            .decrypt()
            .key_id(self.kms_key_id.to_owned())
            .ciphertext_blob(data.to_owned() )
            .send()
            .await?;

        let inner = resp.plaintext.unwrap();
        let bytes = inner.as_ref();

        let secret_text_raw = String::from_utf8(bytes.to_vec())?;

        Ok(secret_text_raw)
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

    async fn get_by_id(&self, user_id: &String) -> ResultE<KeyPair> {
        let _id_av = AttributeValue::S(user_id.to_string());
        let request = self
            .client_dynamo
            .get_item()
            .table_name(KEYPAIRS_TABLE_NAME)
            .key(KEYPAIRS_USER_ID_FIELD_PK, _id_av.clone());

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(KeyPairDynamoDBError(e.to_string()).into());
            }
            Ok(_) => {}
        }
        match results.unwrap().item {
            None => Err(KeyPairNoExistsError("id doesn't exist".to_string()).into()),
            Some(aux) => {
                let mut keypair = KeyPair::new();

                mapping_from_doc_to_keypair(&aux, &mut keypair);

                Ok(keypair)
            }
        }
    }

    async fn get_or_create(&self, user_id: &String) -> ResultE<KeyPair> {
        let get_op = self.get_by_id(user_id).await;
        match get_op {
            Ok(value) => {
                return Ok(value.clone());
            }
            Err(e) => {
                if let Some(_) = e.downcast_ref::<KeyPairDynamoDBError>() {
                    return Err(e);
                } else if let Some(_) = e.downcast_ref::<KeyPairNoExistsError>() {
                    let secp = secp256k1::Secp256k1::new();

                    //let mut rng = rand_hc::Hc128Rng::from_entropy();
                    let mut rng = rngs::StdRng::seed_from_u64(rand::random::<u64>());

                    let contract_owner_key_pair = secp.generate_keypair(&mut rng);
                    let contract_owner_public = contract_owner_key_pair.1.serialize();
                    let hash = keccak256(&contract_owner_public[1..32]);
                    let user_address = format!("0x{}", hex::encode(&hash[12..32]));
                    //let user_private = contract_owner_key_pair.0;
                    let user_private_key =
                        format!("{}", contract_owner_key_pair.0.display_secret());
                    let user_public_key = format!("{}", contract_owner_key_pair.1);

                    let user_private_key_cyphered = self.encrypt(user_private_key).await.unwrap();
                    let user_public_key_cyphered = self.encrypt(user_public_key).await.unwrap();

                    let mut user_key = KeyPair::new();
                    user_key.set_user_id(user_id);
                    user_key.set_address(&user_address);
                    user_key.set_private_key(&user_private_key_cyphered);
                    user_key.set_public_key(&user_public_key_cyphered);

                    self.add(&user_key).await?;

                    return Ok(user_key);
                } else {
                    return Err(KeyPairDynamoDBError(
                        "unexpected issue creating user address".to_string(),
                    )
                    .into());
                }
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
