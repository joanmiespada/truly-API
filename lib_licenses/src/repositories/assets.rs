use std::collections::HashMap;
use std::str::FromStr;

use url::Url;
use uuid::Uuid;

use crate::errors::asset::{AssetDynamoDBError, AssetNoExistsError};
use crate::errors::owner::{OwnerDynamoDBError, OwnerNoExistsError};
use crate::models::asset::{Asset, AssetStatus};
use crate::models::owner::Owner;
use async_trait::async_trait;
use aws_sdk_dynamodb::model::{AttributeValue, Put, Select, TransactWriteItem};
use aws_sdk_dynamodb::Client;
use chrono::{
    prelude::{DateTime, Utc},
    Local,
};
use lib_config::Config;

use super::owners::mapping_from_doc_to_owner;
use super::schema_asset::{ASSETS_TABLE_NAME, ASSET_ID_FIELD_PK};
use super::schema_owners::{OWNERS_TABLE_NAME, OWNER_ASSET_ID_FIELD_PK, OWNER_USER_ID_FIELD_PK};
const URL_FIELD_NAME: &str = "url";
const CREATIONTIME_FIELD_NAME: &str = "creationTime";
const LASTUPDATETIME_FIELD_NAME: &str = "lastUpdateTime";
const STATUS_FIELD_NAME: &str = "assetStatus";

const HASH_FIELD_NAME: &str = "hash";
const LATITUDE_FIELD_NAME: &str = "latitude";
const LONGITUDE_FIELD_NAME: &str = "longitude";
const LICENSE_FIELD_NAME: &str = "license";
const MINTED_FIELD_NAME: &str = "minted";

static NULLABLE: &str = "__NULL__";

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[async_trait]
pub trait AssetRepository {
    async fn add(&self, asset: &mut Asset, user_id: &String) -> ResultE<Uuid>;
    async fn update(&self, id: &Uuid, ass: &Asset) -> ResultE<()>;
    async fn get_by_id(&self, id: &Uuid) -> ResultE<Asset>;
    async fn get_all(&self, page_number: u32, page_size: u32) -> ResultE<Vec<Asset>>;
    async fn get_by_user_id(&self, user_id: &String) -> ResultE<Vec<Asset>>;
    async fn get_by_user_asset_id(&self, asset_id: &Uuid, user_id: &String) -> ResultE<Asset>;
}

#[derive(Clone, Debug)]
pub struct AssetRepo {
    client: Client,
}

impl AssetRepo {
    pub fn new(conf: &Config) -> AssetRepo {
        AssetRepo {
            client: Client::new(conf.aws_config()),
        }
    }
    async fn _get_by_id(&self, id: &Uuid) -> ResultE<HashMap<String, AttributeValue>> {
        let asset_id_av = AttributeValue::S(id.to_string());

        let request = self
            .client
            .get_item()
            .table_name(ASSETS_TABLE_NAME)
            .key(ASSET_ID_FIELD_PK, asset_id_av);

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(AssetDynamoDBError(e.to_string()).into());
            }
            Ok(_) => {}
        }
        match results.unwrap().item {
            None => Err(AssetNoExistsError("id doesn't exist".to_string()).into()),
            Some(aux) => Ok(aux),
        }
    }
}

#[async_trait]
impl AssetRepository for AssetRepo {
    async fn add(&self, asset: &mut Asset, user_id: &String) -> ResultE<Uuid> {
        let asset_id_av = AttributeValue::S(asset.id().to_string());
        let user_id_av = AttributeValue::S(user_id.clone());
        let url_av = AttributeValue::S(asset.url().clone().unwrap().to_string());
        let creation_time_av = AttributeValue::S(iso8601(asset.creation_time()));
        let update_time_av = AttributeValue::S(iso8601(asset.creation_time()));
        let status_av = AttributeValue::S(asset.state().to_string());

        let hash_av = AttributeValue::S(asset.hash().clone().unwrap().to_string());

        let longitude_av;
        match asset.longitude() {
            Some(value) => longitude_av = AttributeValue::S(value.to_string()),
            None => longitude_av = AttributeValue::S(NULLABLE.to_string()),
        }
        let latitude_av;
        match asset.latitude() {
            Some(value) => latitude_av = AttributeValue::S(value.to_string()),
            None => latitude_av = AttributeValue::S(NULLABLE.to_string()),
        }
        let license_av;
        match asset.license() {
            Some(value) => license_av = AttributeValue::S(value.to_string()),
            None => license_av = AttributeValue::S(NULLABLE.to_string()),
        }
        let request = self
            .client
            .transact_write_items()
            .transact_items(
                TransactWriteItem::builder()
                    .put(
                        Put::builder()
                            .item(ASSET_ID_FIELD_PK, asset_id_av.clone())
                            .item(CREATIONTIME_FIELD_NAME, creation_time_av)
                            .item(LASTUPDATETIME_FIELD_NAME, update_time_av)
                            .item(URL_FIELD_NAME, url_av)
                            .item(HASH_FIELD_NAME, hash_av)
                            .item(LONGITUDE_FIELD_NAME, longitude_av)
                            .item(LATITUDE_FIELD_NAME, latitude_av)
                            .item(LICENSE_FIELD_NAME, license_av)
                            .item(STATUS_FIELD_NAME, status_av)
                            .table_name(ASSETS_TABLE_NAME)
                            .build(),
                    )
                    .build(),
            )
            .transact_items(
                TransactWriteItem::builder()
                    .put(
                        Put::builder()
                            .item(OWNER_USER_ID_FIELD_PK, user_id_av)
                            .item(OWNER_ASSET_ID_FIELD_PK, asset_id_av)
                            .table_name(OWNERS_TABLE_NAME)
                            .build(),
                    )
                    .build(),
            );
        /*
        .put_item()
        .table_name(ASSETS_TABLE_NAME)
        .item(ASSET_ID_FIELD_PK, id_av)
        .item(CREATIONTIME_FIELD_NAME, creation_time_av)
        .item(LASTUPDATETIME_FIELD_NAME, update_time_av)
        .item(URL_FIELD_NAME, url_av)
        .item(HASH_FIELD_NAME, hash_av)
        .item(LONGITUDE_FIELD_NAME, longitude_av)
        .item(LATITUDE_FIELD_NAME, latitude_av)
        .item(LICENSE_FIELD_NAME, license_av)
        .item(STATUS_FIELD_NAME, status_av);*/

        match request.send().await {
            Ok(_) => Ok(asset.id().clone()),
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(AssetDynamoDBError(e.to_string()).into());
            }
        }
    }

    async fn get_all(&self, _page_number: u32, _page_size: u32) -> ResultE<Vec<Asset>> {
        let mut queried = Vec::new();

        let results = self
            .client
            .scan()
            .table_name(ASSETS_TABLE_NAME)
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
                return Err(AssetDynamoDBError(e.to_string()).into());
            }
            Ok(result) => {
                if let Some(docs) = result.items {
                    for doc in docs {
                        let mut asset = Asset::new();

                        mapping_from_doc_to_asset(&doc, &mut asset);

                        queried.push(asset.clone());
                    }
                }
            }
        }

        Ok(queried)
    }

    async fn get_by_id(&self, id: &Uuid) -> ResultE<Asset> {
        let res = self._get_by_id(id).await?;
        let mut asset = Asset::new();
        mapping_from_doc_to_asset(&res, &mut asset);
        Ok(asset)
    }

    async fn update(&self, id: &Uuid, asset: &Asset) -> ResultE<()> {
        let last_update_time_av = AttributeValue::S(iso8601(&Utc::now()));
        let id_av = AttributeValue::S(id.to_string());

        let url = asset.url().clone().unwrap().to_string();
        let url_av = AttributeValue::S(url);

        let status_av: AttributeValue = AttributeValue::S(asset.state().to_string());

        let hash_av = AttributeValue::S(asset.hash().clone().unwrap().to_string());

        let longitude_av;
        match asset.longitude() {
            Some(value) => longitude_av = AttributeValue::S(value.to_string()),
            None => longitude_av = AttributeValue::S(NULLABLE.to_string()),
        }
        let latitude_av;
        match asset.latitude() {
            Some(value) => latitude_av = AttributeValue::S(value.to_string()),
            None => latitude_av = AttributeValue::S(NULLABLE.to_string()),
        }
        let license_av;
        match asset.license() {
            Some(value) => license_av = AttributeValue::S(value.to_string()),
            None => license_av = AttributeValue::S(NULLABLE.to_string()),
        }
        let minted_tx_av;
        match asset.minted_tx() {
            Some(value) => minted_tx_av = AttributeValue::S(value.to_string()),
            None => minted_tx_av = AttributeValue::S(NULLABLE.to_string()),
        }

        let mut update_express = "set ".to_string();
        update_express.push_str(format!("{0} = :url, ", URL_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :lastup, ", LASTUPDATETIME_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :_status ", STATUS_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :hash ", HASH_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :longitude ", LONGITUDE_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :latitude ", LATITUDE_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :license ", LICENSE_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :minted_tx ", MINTED_FIELD_NAME).as_str());

        let request = self
            .client
            .update_item()
            .table_name(ASSETS_TABLE_NAME)
            .key(ASSET_ID_FIELD_PK, id_av)
            .update_expression(update_express)
            .expression_attribute_values(":url", url_av)
            .expression_attribute_values(":lastup", last_update_time_av)
            .expression_attribute_values(":hash", hash_av)
            .expression_attribute_values(":longitude", longitude_av)
            .expression_attribute_values(":latitude", latitude_av)
            .expression_attribute_values(":license", license_av)
            .expression_attribute_values(":minted_tx", minted_tx_av)
            .expression_attribute_values(":_status", status_av);

        match request.send().await {
            Ok(_) => Ok(()),
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(AssetDynamoDBError(e.to_string()).into());
            }
        }
    }

    async fn get_by_user_id(&self, user_id: &String) -> ResultE<Vec<Asset>> {
        let mut queried = Vec::new();
        let user_id_av = AttributeValue::S(user_id.to_string());

        let mut filter = "".to_string();
        filter.push_str(OWNER_USER_ID_FIELD_PK);
        filter.push_str(" = :value");

        let request = self
            .client
            .query()
            .table_name(OWNERS_TABLE_NAME)
            .key_condition_expression(filter)
            .expression_attribute_values(":value".to_string(), user_id_av);
            //.select(Select::AllProjectedAttributes);
        //.key(OWNER_USER_ID_FIELD_PK, _id_av.clone());

        let mut assets_list = Vec::new();
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
            Ok(data) => {
                let op_items = data.items();
                match op_items {
                    None => {
                        return Err(OwnerNoExistsError("id doesn't exist".to_string()).into());
                    }
                    Some(aux) => {
                        for doc in aux {
                            let mut own = Owner::new();
                            mapping_from_doc_to_owner(&doc, &mut own);
                            assets_list.push(own.clone());
                        }
                    }
                }
            }
        }
        

        for ass in assets_list {
            let res = self._get_by_id(ass.asset_id()).await?;
            let mut asset = Asset::new();
            mapping_from_doc_to_asset(&res, &mut asset);
            queried.push(asset.clone());
        }
        Ok(queried)
    }

    async fn get_by_user_asset_id(&self, asset_id: &Uuid, user_id: &String) -> ResultE<Asset> {
        let user_id_av = AttributeValue::S(user_id.to_string());
        let asset_id_av = AttributeValue::S(asset_id.to_string());

        // let mut filter = "".to_string();
        // filter.push_str(OWNER_USER_ID_FIELD_PK);
        // filter.push_str(" = :User_id_value, ");
        // filter.push_str(OWNER_ASSET_ID_FIELD_PK);
        // filter.push_str(" = :asset_id_value");
        let mut filter2 = HashMap::new();
        filter2.insert(OWNER_USER_ID_FIELD_PK.to_string(), user_id_av);
        filter2.insert(OWNER_ASSET_ID_FIELD_PK.to_string(), asset_id_av);

        // let mut filter = HashMap::new();
        // filter.insert(":v1".to_string(), user_id_av.clone());
        // filter.insert(":v2".to_string(), asset_id_av.clone());
        // let express = format!("{} = :v1 and {} = :v2",OWNER_USER_ID_FIELD_PK,OWNER_ASSET_ID_FIELD_PK);


        let request = self
            .client
            //.query()
            .get_item()
            .table_name(OWNERS_TABLE_NAME)
            .set_key(Some(filter2));
            //.set_key_condition_expression(Some(express))
            //.set_expression_attribute_names(input)
            //.set_expression_attribute_values(Some(filter));
            //.set_key(Some(filter));
            //.key_condition_expression(filter);
            
            //.expression_attribute_values(":v1".to_string(), user_id_av)
            //.expression_attribute_values(":v2".to_string(), asset_id_av);
            //.key(OWNER_USER_ID_FIELD_PK, user_id_av.clone());
            //.key(OWNER_ASSET_ID_FIELD_PK, asset_id_av.clone());

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

        let mut own = Owner::new();
        match results.unwrap().item {
            None => {
                return Err(OwnerNoExistsError("owner doesn't exist".to_string()).into());
            }
            Some(aux) => {
                mapping_from_doc_to_owner(&aux, &mut own);
            }
        }
        let res = self._get_by_id(own.asset_id()).await?;
        let mut asset = Asset::new();
        mapping_from_doc_to_asset(&res, &mut asset);
        Ok(asset)
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
fn mapping_from_doc_to_asset(doc: &HashMap<String, AttributeValue>, asset: &mut Asset) {
    let _id = doc.get(ASSET_ID_FIELD_PK).unwrap();
    let asset_id = _id.as_s().unwrap();
    let uuid = Uuid::from_str(asset_id).unwrap();
    asset.set_id(&uuid);

    let _url = doc.get(URL_FIELD_NAME).unwrap();
    let asset_url = _url.as_s().unwrap();
    let url = Url::parse(asset_url).unwrap();
    asset.set_url(&Some(url));

    let _hash = doc.get(HASH_FIELD_NAME).unwrap();
    let asset_hash = _hash.as_s().unwrap();
    asset.set_hash(&Some(asset_hash.to_string()));

    let creation_time_t = doc.get(CREATIONTIME_FIELD_NAME);
    match creation_time_t {
        None => {}
        Some(creation_time) => {
            asset.set_creation_time(&from_iso8601(creation_time.as_s().unwrap()));
        }
    }

    let last_update_time_t = doc.get(LASTUPDATETIME_FIELD_NAME);
    match last_update_time_t {
        None => {}
        Some(last_update_time) => {
            asset.set_last_update_time(&from_iso8601(last_update_time.as_s().unwrap()));
        }
    }

    let longitude_t = doc.get(LONGITUDE_FIELD_NAME);
    match longitude_t {
        None => asset.set_longitude(&None),
        Some(longi) => {
            let val = longi.as_s().unwrap();
            if val == NULLABLE {
                asset.set_longitude(&None)
            } else {
                let f_val = f64::from_str(val);
                match f_val {
                    Err(_) => asset.set_longitude(&None),
                    Ok(final_number) => asset.set_longitude(&Some(final_number)),
                }
            }
        }
    }

    let latitude_t = doc.get(LATITUDE_FIELD_NAME);
    match latitude_t {
        None => asset.set_latitude(&None),
        Some(lati) => {
            let val = lati.as_s().unwrap();
            if val == NULLABLE {
                asset.set_latitude(&None)
            } else {
                let f_val = f64::from_str(val);
                match f_val {
                    Err(_) => asset.set_latitude(&None),
                    Ok(final_number) => asset.set_latitude(&Some(final_number)),
                }
            }
        }
    }

    let license_t = doc.get(LICENSE_FIELD_NAME);
    match license_t {
        None => asset.set_license(&None),
        Some(lati) => {
            let val = lati.as_s().unwrap();
            if val == NULLABLE {
                asset.set_license(&None)
            } else {
                asset.set_license(&Some(val.clone()))
            }
        }
    }

    let status_t = doc.get(STATUS_FIELD_NAME).unwrap().as_s().unwrap();
    let aux = AssetStatus::from_str(status_t).unwrap();
    asset.set_state(&aux);
}
