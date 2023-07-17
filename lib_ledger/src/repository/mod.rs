use self::schema_ledger::{
    LEDGER_FIELD_ASSET_ID, LEDGER_FIELD_HASH, LEDGER_NAME, LEDGER_TABLE_NAME,
};
use crate::errors::LedgerError;
use crate::models::{AssetLedged, Ledge};
use async_trait::async_trait;
use base64::engine::general_purpose;
use base64::Engine;
use chrono::{prelude::Utc, DateTime};
use lib_config::config::Config;
use lib_config::result::ResultE;
use lib_licenses::models::asset::Asset;
use qldb::{ion::IonValue, Document, QldbClient};
use uuid::Uuid;
pub mod schema_ledger;

#[async_trait]
pub trait LedgerRepository {
    async fn add(&self, asset: &Asset) -> ResultE<Ledge>;
    async fn get_by_hash(&self, hash: &String) -> ResultE<Ledge>;
    async fn get_by_asset_id(&self, asset_id: &Uuid) -> ResultE<Ledge>;
}

#[derive(Clone, Debug)]
pub struct LedgerRepo {
    //client: Client,
    //region: String,
    //endpoint: String,
}

impl LedgerRepo {
    pub fn new(_conf: &Config) -> LedgerRepo {
        LedgerRepo {
            //client: Client::new(conf.aws_config()),
            //region: conf.env_vars().aws_region().unwrap(),
            //endpoint: conf.env_vars().aws_endpoint().unwrap(),
        }
    }

    fn asset_to_ledged(asset: &Asset) -> AssetLedged {
        AssetLedged {
            asset_id: asset.id().clone(),
            asset_hash: asset.hash().clone().unwrap(),
            asset_hash_algorithm: asset.hash_algorithm().clone().unwrap(),
            asset_creation_time: Utc::now(),
        }
    }
}

#[async_trait]
impl LedgerRepository for LedgerRepo {
    async fn add(&self, asset: &Asset) -> ResultE<Ledge> {
        let asset = LedgerRepo::asset_to_ledged(asset);
        let data = asset.to_hash_map();

        let client = QldbClient::default(LEDGER_NAME, 200).await?;
        let op: Result<Option<String>, qldb::QldbError> = client
            .transaction_within(|client| async move {
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
                    .execute()
                    .await;

                let result = result.unwrap();

                let first_count = match result[0].get("_1") {
                    Some(IonValue::Integer(count)) => count.to_owned(),
                    _ => panic!("First count returned a non integer"),
                };

                if first_count != 0 {
                    client.rollback().await?;
                    return Ok(None);
                } else {
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
                            doc_id = id;
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

                    let qldb_committed = map_to_ledge(value);
                    Ok(qldb_committed)
                }
            }
        }
    }
    async fn get_by_hash(&self, hash: &String) -> ResultE<Ledge> {
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

        let qldb_committed = map_to_ledge(value);
        Ok(qldb_committed)
    }
    async fn get_by_asset_id(&self, asset_id: &Uuid) -> ResultE<Ledge> {
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

        let qldb_committed = map_to_ledge(value);
        Ok(qldb_committed)
    }
}

pub fn map_to_ledge(map: Document) -> Ledge {
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
