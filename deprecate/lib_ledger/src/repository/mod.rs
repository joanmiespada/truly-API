use std::collections::HashMap;
use std::str::FromStr;

use self::schema_ledger::{
    LEDGER_FIELD_ASSET_ID, LEDGER_FIELD_HASH, LEDGER_NAME, LEDGER_TABLE_NAME, DYNAMODB_TABLE_NAME, DYNAMODB_ASSET_ID_FIELD_PK, DYNAMODB_HASH_FIELD_NAME, DYNAMODB_TABLE_INDEX_HASH,
};
use crate::errors::{LedgerError, LedgerDynamodbError};
use crate::models::{AssetLedged, Ledge};
use async_trait::async_trait;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::types::{AttributeValue, Select};
use base64::engine::general_purpose;
use base64::Engine;
use chrono::Local;
use chrono::{prelude::Utc, DateTime};
use lib_config::config::Config;
use lib_config::result::ResultE;
use lib_config::timing::from_iso8601;
use qldb::{ion::IonValue, Document, QldbClient};
use tracing::info;
use uuid::Uuid;
pub mod schema_ledger;

const DYNAMODB_CREATIONTIME_FIELD_NAME: &str = "creationTime";
const DYNAMODB_BLOCKCHAIN_ID_FIELD_NAME: &str = "blockchainId";
const DYNAMODB_BLOCKCHAIN_SEQ_FIELD_NAME: &str = "blockchainSeq";
const DYNAMODB_META_ID_FIELD_NAME:&str = "metaID";
const DYNAMODB_META_TX_ID_FIELD_NAME:&str = "metaTX";
const DYNAMODB_VERSION_FIELD_NAME:&str = "metaVer";

#[async_trait]
pub trait LedgerRepository {
    async fn add(&self, asset: &AssetLedged) -> ResultE<Ledge>;
    async fn ledger_get_by_hash(&self, hash: &String) -> ResultE<Ledge>;
    async fn ledger_get_by_asset_id(&self, asset_id: &Uuid) -> ResultE<Ledge>;

    async fn get_by_hash(&self, hash: &String) -> ResultE<Option<Ledge>>;
    async fn get_by_asset_id(&self, asset_id: &Uuid) -> ResultE<Option<Ledge>>;
}

#[derive(Clone, Debug)]
pub struct LedgerRepo {
    client: Client,
}

impl LedgerRepo {
    pub fn new(conf: &Config) -> LedgerRepo {
        LedgerRepo {
            client: Client::new(conf.aws_config()),
        }
    }

    async fn add_qldb(&self, asset: &AssetLedged) -> ResultE<Ledge> {
        let data = asset.to_hash_map();

       info!("calling ledger service: getting qldb session");
       let client = QldbClient::default(LEDGER_NAME, 200).await?;
       info!("calling ledger service: check if asset id already exist");
        let op: Result<Option<String>, qldb::QldbError> = client
            .transaction_within(|client| async move {
                info!("calling now!!!");
                let doc_id;
                let result = client
                    .query(
                        format!(
                            "SELECT COUNT(asset_id) FROM {} WHERE {} = ?",
                            LEDGER_TABLE_NAME, LEDGER_FIELD_ASSET_ID
                        )
                        .as_str(),
                    )
                    .param(asset.asset_id.to_string())
                    .count()
                    //.execute()
                    .await;
                info!("called!!!");
                let first_count_op = result;
                let first_count;
                match first_count_op {
                    Ok(val) => first_count=val,
                    Err(e)=>{
                        panic!("{}",e)
                    }
                    
                }
                info!("response! {}", first_count);

                //let first_count = match result //[0].get("_1") {
                //    Some(IonValue::Integer(count)) => count.to_owned(),
                //    _ => panic!("First count returned a non integer"),
                //};

                if first_count != 0 {
                    client.rollback().await?;
                    return Ok(None);
                } else {
                    info!("inserting a new one!!!");
                    let sentence = format!("INSERT INTO {} VALUE ?", LEDGER_TABLE_NAME);
                    //println!("{}", sentence);
                    let op = client.query(&sentence.as_str()).param(data).execute().await;
                    match op {
                        Err(e) => {
                            panic!("{}", e);
                            //Err(e)
                        }
                        Ok(op) => {
                            let id = op[0].get("documentId").clone();
                            let id = match id {
                                Some(IonValue::String(id)) => id.to_owned(),
                                _ => panic!("DocumentID returned non String"),
                            };
                            //println!("documentId created: {}", id);
                            doc_id = id.to_owned();
                            info!("calling ledger service: asset id created successfully at QLDB service, documnet ID: {}", id );
                        }
                    }
                    //client.commit().await?;
                }
                Ok(Some(doc_id))
            }) 
            .await;

        if let Err(e) = op {
            let message = format!(
                "Error at ledger, asset id could be already stored: {} e: {}",
                asset.asset_id, e
            );
            return Err(Box::new(LedgerError { 0: message }));
        } else {
            match op.ok().unwrap() {
                None => {
                    return Err(Box::new(LedgerError {0:"Error creating doc at ledger".to_string() }));
                },
                Some(doc_id) => {
                    info!("getting blockchain information...");
                    let sentence = format!(
                        r#"SELECT * FROM _ql_committed_{table} as r  WHERE  r.metadata.id = '{doc}' "#,
                        table = LEDGER_TABLE_NAME,
                        doc = doc_id
                    );
                    //println!("{}", sentence);
                    let stataux = client.read_query(&sentence).await?.execute().await.unwrap();

                    if stataux.len() != 1 {
                        let message = format!(
                            "asset id not found in blockchain metadata: {}",
                            asset.asset_id
                        );
                        return Err(Box::new(LedgerError { 0: message }));
                    }
                    let value = stataux[0].clone();

                    let qldb_committed = qldb_map_to_ledge(value);
                    info!("blockchain data got it");
                    Ok(qldb_committed)
                }
            }
        }
    }
    async fn add_dynamodb(&self, asset_id: &Uuid, ledge: &Ledge) -> ResultE<()> {

        let asset_id_av = AttributeValue::S( asset_id.to_string());
        let creation_time_av = AttributeValue::S( ledge.metadata_tx_time.to_rfc3339());
        let blockchain_id_av = AttributeValue::S( ledge.blockchain_id.to_owned());
        let blockchain_seq_av = AttributeValue::N( ledge.blockchain_seq.to_string());
        let hash_av = AttributeValue::S( ledge.hash.to_owned());
        let meta_id_av = AttributeValue::S( ledge.metadata_id.to_owned());
        let meta_tx_id_av = AttributeValue::S( ledge.metadata_tx_id.to_owned());
        let meta_version_av = AttributeValue::N( ledge.metadata_version.to_string());

        let request = self
            .client
            .put_item()
            .table_name( DYNAMODB_TABLE_NAME)
            .item(DYNAMODB_ASSET_ID_FIELD_PK, asset_id_av)
            .item(DYNAMODB_CREATIONTIME_FIELD_NAME, creation_time_av)
            .item(DYNAMODB_BLOCKCHAIN_ID_FIELD_NAME, blockchain_id_av)
            .item(DYNAMODB_BLOCKCHAIN_SEQ_FIELD_NAME, blockchain_seq_av)
            .item(DYNAMODB_HASH_FIELD_NAME, hash_av)
            .item(DYNAMODB_META_ID_FIELD_NAME, meta_id_av )
            .item(DYNAMODB_META_TX_ID_FIELD_NAME, meta_tx_id_av )
            .item(DYNAMODB_VERSION_FIELD_NAME, meta_version_av);

        match request.send().await {
            Ok(_) => Ok(()),
            Err(e) => {
                let message = format!("Error storing blockchain status: {}", e);
                tracing::error!("{}", message);
                Err(LedgerDynamodbError(message).into())
            }
        }
    
    }
}

#[async_trait]
impl LedgerRepository for LedgerRepo {
    async fn add(&self, asset: &AssetLedged) -> ResultE<Ledge> {

       info!("calling ledger service: adding qldb");
       let l = self.add_qldb(asset).await?; 
       info!("calling ledger service: adding dynamodb");
       self.add_dynamodb( &asset.asset_id, &l).await?;
       Ok(l)
    }
    async fn ledger_get_by_hash(&self, hash: &String) -> ResultE<Ledge> {
        let client = QldbClient::default(LEDGER_NAME, 200).await?;

        let stataux = client
            .read_query(&format!(
                r#"SELECT * FROM _ql_committed_{} as r WHERE r.data.{} = '{}' "#,
                LEDGER_TABLE_NAME, LEDGER_FIELD_HASH, hash
            ))
            .await?
            .execute()
            .await
            .unwrap();

        let value = stataux[0].clone();

        let qldb_committed = qldb_map_to_ledge(value);
        Ok(qldb_committed)
    }
    async fn ledger_get_by_asset_id(&self, asset_id: &Uuid) -> ResultE<Ledge> {
        let client = QldbClient::default(LEDGER_NAME, 200).await?;

        let stataux = client
            .read_query(&format!(
                r#"SELECT * FROM _ql_committed_{} as r WHERE  r.data.{} = '{}' "#,
                LEDGER_TABLE_NAME,
                LEDGER_FIELD_ASSET_ID,
                asset_id.to_string()
            ))
            .await?
            .execute()
            .await
            .unwrap();

        let value = stataux[0].clone();

        let qldb_committed = qldb_map_to_ledge(value);
        Ok(qldb_committed)
    }
    async fn get_by_hash(&self, hash: &String) -> ResultE<Option<Ledge>>
    {
        let asset_hash_av = AttributeValue::S(hash.to_string());

        let mut filter = "".to_string();
        filter.push_str(DYNAMODB_HASH_FIELD_NAME);
        filter.push_str(" = :value");

        let request = self
            .client
            .query()
            .table_name(DYNAMODB_TABLE_NAME)
            .index_name(DYNAMODB_TABLE_INDEX_HASH)
            .key_condition_expression(filter)
            .expression_attribute_values(":value".to_string(), asset_hash_av)
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
                return Err(LedgerDynamodbError(e.to_string()).into());
            }
            Ok(data) => {
                let op_items = data.items();
                match op_items {
                    None => {
                        Ok(None)
                    }
                    Some(aux) => {
                        if aux.len() == 0 {
                            Ok(None)
                        } else {
                            let doc = aux.first().unwrap().to_owned();
                            let assetid= doc.get(DYNAMODB_ASSET_ID_FIELD_PK).unwrap().to_owned().as_s().unwrap().to_string();
                            let asset_id = Uuid::from_str(&assetid).unwrap();
                            let l = self.get_by_asset_id(&asset_id).await?;
                            Ok(l)
                        }
                    }
                }
            }
        }
    }
    async fn get_by_asset_id(&self, asset_id: &Uuid) -> ResultE<Option<Ledge>>
    {
        let asset_id_av = AttributeValue::S(asset_id.to_string());

        let request = self
            .client
            .get_item()
            .table_name(DYNAMODB_TABLE_NAME)
            .key(DYNAMODB_ASSET_ID_FIELD_PK, asset_id_av);

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(LedgerDynamodbError(e.to_string()).into());
            }
            Ok(_) => {}
        }
        match results.unwrap().item {
            None => Ok(None),
            Some(aux) => { 
                let l = dynamodb_map_to_ledge(aux);  
                Ok(Some(l))
            },
        }

    }
}

pub fn dynamodb_map_to_ledge(doc: HashMap<String, AttributeValue>) -> Ledge {

    let mut result: Ledge = Ledge::default();

    let creation_time_op = doc.get(DYNAMODB_CREATIONTIME_FIELD_NAME);
    if let Some(creation_time) =  creation_time_op {
            result.metadata_tx_time = from_iso8601(creation_time.as_s().unwrap());
    }
    let blockchain_id_op = doc.get(DYNAMODB_BLOCKCHAIN_ID_FIELD_NAME);
    if let Some(blockchain_id) =  blockchain_id_op {
            result.blockchain_id = blockchain_id.as_s().unwrap().to_string();
    }
    let blockchain_seq_op = doc.get(DYNAMODB_BLOCKCHAIN_SEQ_FIELD_NAME);
    if let Some(blockchain_seq) =  blockchain_seq_op {
            let val = blockchain_seq.as_n().unwrap().to_string();
            let f_val = u64::from_str_radix(&val, 10).unwrap();
            result.blockchain_seq = f_val;
    }
    let hash_op = doc.get(DYNAMODB_HASH_FIELD_NAME);
    if let Some(value) =  hash_op {
            result.hash = value.as_s().unwrap().to_string();
    }
    let meta_id_op = doc.get(DYNAMODB_META_ID_FIELD_NAME);
    if let Some(value) =  meta_id_op {
            result.metadata_id = value.as_s().unwrap().to_string();
    }
    let meta_tx_op = doc.get(DYNAMODB_META_TX_ID_FIELD_NAME);
    if let Some(value) =  meta_tx_op {
            result.metadata_tx_id = value.as_s().unwrap().to_string();
    }
    let meta_version_op = doc.get(DYNAMODB_VERSION_FIELD_NAME);
    if let Some(value) =  meta_version_op {
            let val = value.as_n().unwrap().to_string();
            let f_val = u64::from_str_radix(&val, 10).unwrap();
            result.metadata_version = f_val;
    }
    result
}

pub fn qldb_map_to_ledge(map: Document) -> Ledge {
    let block_address = map.get("blockAddress").clone();
    let block_address = match block_address {
        Some(IonValue::Struct(block_address)) => block_address,
        _ => panic!("Block Address must be present"),
    };

    let _strand_id = block_address.get("strandId").clone();
    let strand_id = match _strand_id {
        Some(IonValue::String(value)) => value.to_owned(),
        _ => panic!("Strand ID returned a non integer"),
    };
    let _sequence_num = block_address.get("sequenceNo").clone();
    let sequence_num = match _sequence_num {
        Some(IonValue::Integer(count)) => count.to_owned(),
        _ => panic!("Sequence Num returned a non integer"),
    };

    let _hash = map.get("hash").clone();
    let hash = match _hash {
        Some(IonValue::Blob(value)) => value.to_owned(),
        _ => panic!("Hash returned a non String"),
    };

    let hash = general_purpose::STANDARD.encode(&hash);

    let _metadata = map.get("metadata").clone();
    let metadata = match _metadata {
        Some(IonValue::Struct(meta)) => meta,
        _ => panic!("Metadata must be present"),
    };
    let _id = metadata.get("id").clone();
    let id = match _id {
        Some(IonValue::String(id)) => id.to_owned(),
        _ => panic!("Metadata id returned non String"),
    };
    let _version = metadata.get("version").clone();
    let version = match _version {
        Some(IonValue::Integer(ver)) => ver.to_owned(),
        _ => panic!("Metadata version returned a non integer"),
    };
    let _tx_time = metadata.get("txTime").clone();
    let tx_time: DateTime<Utc> = match _tx_time {
        Some(IonValue::DateTime(txx)) => txx.to_owned().into(),
        _ => panic!("tx time returned a non String"),
    };

    let tx_time = tx_time.with_timezone(&Utc);

    let _tx_id = metadata.get("txId").clone();
    let tx_id = match _tx_id {
        Some(IonValue::String(txx)) => txx.to_owned(),
        _ => panic!("tx Id returned a non String"),
    };

    Ledge {
        metadata_version: version as u64,
        metadata_tx_id: tx_id,
        metadata_id: id,
        metadata_tx_time: tx_time,
        hash,
        blockchain_id: strand_id,
        blockchain_seq: sequence_num as u64,
    }


    
}
