use std::collections::HashMap;
use std::str::FromStr;

use uuid::Uuid;

use crate::errors::owner::{OwnerDynamoDBError, OwnerNoExistsError};
use crate::models::owner::Owner;
use async_trait::async_trait;
use aws_sdk_dynamodb::{model::AttributeValue, Client};
use chrono::{
    prelude::{DateTime, Utc},
    Local,
};
use lib_config::Config;

use super::schema_owners::{OWNER_ASSET_ID_FIELD_PK, OWNERS_TABLE_NAME, OWNER_USER_ID_FIELD_PK};
pub const CREATIONTIME_FIELD_NAME: &str = "creationTime";
pub const LASTUPDATETIME_FIELD_NAME: &str = "lastUpdateTime";


type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[async_trait]
pub trait OwnerRepository {
    async fn add(&self, owner: &mut Owner) -> ResultE<()>;
    async fn update(&self, old_owner: &Owner, new_owner: &String) -> ResultE<()>;
    async fn get_by_asset(&self, asset_id: &Uuid) -> ResultE<Owner>;
    async fn get_by_user(&self, user_id: &String) -> ResultE<Vec<Owner>>;
    async fn get_by_user_asset(&self, asset_id: &Uuid, user_id: &String) -> ResultE<Owner>;
    async fn get_all(&self, page_number: u32, page_size: u32) -> ResultE<Vec<Owner>>;
}

#[derive(Clone, Debug)]
pub struct OwnerRepo {
    client: Client,
}

impl OwnerRepo {
    pub fn new(conf: &Config) -> OwnerRepo {
        OwnerRepo {
            client: Client::new(conf.aws_config()),
        }
    }
}

#[async_trait]
impl OwnerRepository for OwnerRepo {
    async fn add(&self, owner: &mut Owner) -> ResultE<()> {
        let user_id_av = AttributeValue::S(owner.user_id().to_string());
        let asset_id_av = AttributeValue::S(owner.asset_id().to_string());
        let creation_time_av = AttributeValue::S(iso8601(owner.creation_time()));
        let update_time_av = AttributeValue::S(iso8601(owner.creation_time()));

        let request = self
            .client
            .put_item()
            .table_name(OWNERS_TABLE_NAME)
            .item(OWNER_USER_ID_FIELD_PK, user_id_av)
            .item(OWNER_ASSET_ID_FIELD_PK, asset_id_av)
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
                return Err(OwnerDynamoDBError(e.to_string()).into());
            }
        }
    }

    async fn get_all(&self, _page_number: u32, _page_size: u32) -> ResultE<Vec<Owner>> {
        let mut queried = Vec::new();

        let results = self
            .client
            .scan()
            .table_name(OWNERS_TABLE_NAME)
            .send()
            .await;

        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(OwnerDynamoDBError(e.to_string()).into());
            }
            Ok(result) => {
                if let Some(docs) = result.items {
                    for doc in docs {
                        let mut owner = Owner::new();

                        mapping_from_doc_to_owner(&doc, &mut owner);

                        queried.push(owner.clone());
                    }
                }
            }
        }

        Ok(queried)
    }

    async fn get_by_user(&self, id: &String) -> ResultE<Vec<Owner>> {
        let mut queried = Vec::new();
        let _id_av = AttributeValue::S(id.to_string());
        let request = self
            .client
            .get_item()
            .table_name(OWNERS_TABLE_NAME)
            .key(OWNER_USER_ID_FIELD_PK, _id_av.clone());

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(OwnerDynamoDBError(e.to_string()).into());
            }
            Ok(_) => {}
        }
        match results.unwrap().item {
            None => Err(OwnerNoExistsError("id doesn't exist".to_string()).into()),
            Some(aux) => {
                // for doc in docs {
                let mut owner = Owner::new();

                mapping_from_doc_to_owner(&aux, &mut owner);

                queried.push(owner.clone());
                // }
                Ok(queried)
            }
        }
    }

    async fn get_by_asset(&self, id: &Uuid) -> ResultE<Owner> {
        let _id_av = AttributeValue::S(id.to_string());
        let request = self
            .client
            .get_item()
            .table_name(OWNERS_TABLE_NAME)
            .key(OWNER_USER_ID_FIELD_PK, _id_av.clone());

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(OwnerDynamoDBError(e.to_string()).into());
            }
            Ok(_) => {}
        }
        match results.unwrap().item {
            None => Err(OwnerNoExistsError("id doesn't exist".to_string()).into()),
            Some(aux) => {
                // for doc in docs {
                let mut owner = Owner::new();

                mapping_from_doc_to_owner(&aux, &mut owner);

                // }
                Ok(owner)
            }
        }
    }

    async fn get_by_user_asset(&self, asset_id: &Uuid, user_id: &String) -> ResultE<Owner>{
        let asset_id_av = AttributeValue::S(asset_id.to_string());
        let user_id_av = AttributeValue::S(user_id.clone());
        let request = self
            .client
            .get_item()
            .table_name(OWNERS_TABLE_NAME)
            .key(OWNER_USER_ID_FIELD_PK, user_id_av.clone())
            .key(OWNER_ASSET_ID_FIELD_PK, asset_id_av.clone());

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(OwnerDynamoDBError(e.to_string()).into());
            }
            Ok(_) => {}
        }
        match results.unwrap().item {
            None => Err(OwnerNoExistsError("id doesn't exist".to_string()).into()),
            Some(val) => {
                let mut owner = Owner::new();

                mapping_from_doc_to_owner(&aux, &mut owner);

                Ok(owner)

            }
        }
    }

    async fn update(&self, old_owner: &Owner, new_owner: &String) -> ResultE<()> {
        let last_update_time_av = AttributeValue::S(iso8601(&Utc::now()));
        let asset_id_av = AttributeValue::S(old_owner.asset_id().to_string());
        let user_id_av = AttributeValue::S(old_owner.user_id().clone());
        let new_owner_id_av = AttributeValue::S(new_owner.clone());
        let mut update_express = "set ".to_string();
        update_express.push_str(format!("{0} = :new_owner, ", OWNER_USER_ID_FIELD_PK).as_str());
        update_express.push_str(format!("{0} = :lastup, ", LASTUPDATETIME_FIELD_NAME).as_str());

        let request = self
            .client
            .update_item()
            .table_name(OWNERS_TABLE_NAME)
            .key(OWNER_USER_ID_FIELD_PK, user_id_av)
            .key(OWNER_ASSET_ID_FIELD_PK, asset_id_av)
            .update_expression(update_express)
            .expression_attribute_values(":lastup", last_update_time_av)
            .expression_attribute_values(":new_owner", new_owner_id_av);

        match request.send().await {
            Ok(_) => {
                //let mut ow = Owner::new();
                //ow.set_asset_id(old_owner.asset_id());
                //ow.set_user_id(new_owner );
                Ok(())
            },
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(OwnerDynamoDBError(e.to_string()).into());
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
fn mapping_from_doc_to_owner(doc: &HashMap<String, AttributeValue>, owner: &mut Owner) {
    let user_id = doc.get(OWNER_USER_ID_FIELD_PK).unwrap();
    let user_id = user_id.as_s().unwrap();
    //let uuid = Uuid::from_str(owner_id).unwrap();
    owner.set_user_id(&user_id);

    let _asset_id = doc.get(OWNER_ASSET_ID_FIELD_PK).unwrap();
    let asset_id = _asset_id.as_s().unwrap();
    let uuid = Uuid::from_str(asset_id).unwrap();
    owner.set_asset_id(&uuid);

    let creation_time_t = doc.get(CREATIONTIME_FIELD_NAME);
    match creation_time_t {
        None => {}
        Some(creation_time) => {
            owner.set_creation_time(&from_iso8601(creation_time.as_s().unwrap()));
        }
    }

    let last_update_time_t = doc.get(LASTUPDATETIME_FIELD_NAME);
    match last_update_time_t {
        None => {}
        Some(last_update_time) => {
            owner.set_last_update_time(&from_iso8601(last_update_time.as_s().unwrap()));
        }
    }
}
