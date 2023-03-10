use std::collections::HashMap;
use std::str::FromStr;

use tracing::error;
use url::Url;
use uuid::Uuid;
use web3::types::H256;

use crate::errors::asset::{AssetDynamoDBError, AssetNoExistsError};
use crate::errors::owner::{OwnerDynamoDBError, OwnerNoExistsError};
use crate::models::asset::{Asset, AssetStatus, MintingStatus, VideoLicensingStatus};
use crate::models::owner::Owner;
use async_trait::async_trait;
use aws_sdk_dynamodb::model::{AttributeValue, Put, TransactWriteItem};
use aws_sdk_dynamodb::Client;
use chrono::{
    prelude::{DateTime, Utc},
    Local,
};
use lib_config::config::Config;

use super::owners::mapping_from_doc_to_owner;
use super::schema_asset::{
    ASSETS_TABLE_NAME, ASSET_ID_FIELD_PK, ASSET_TREE_FATHER_ID_FIELD, ASSET_TREE_SON_ID_FIELD_PK,
    ASSET_TREE_TABLE_NAME, URL_FIELD_NAME,
};
use super::schema_owners::{OWNERS_TABLE_NAME, OWNER_ASSET_ID_FIELD_PK, OWNER_USER_ID_FIELD_PK};
const CREATIONTIME_FIELD_NAME: &str = "creationTime";
const LASTUPDATETIME_FIELD_NAME: &str = "lastUpdateTime";
const STATUS_FIELD_NAME: &str = "assetStatus";

const HASH_FIELD_NAME: &str = "hash_uri";
const LATITUDE_FIELD_NAME: &str = "latitude";
const LONGITUDE_FIELD_NAME: &str = "longitude";
const LICENSE_FIELD_NAME: &str = "license";
const MINTED_FIELD_NAME: &str = "minted";
const MINTED_STATUS_FIELD_NAME: &str = "minting_status";

const COUNTER_FIELD_NAME: &str = "global_counter";
const SHORTER_FIELD_NAME: &str = "shorter";
const VIDEO_LICENSING_FIELD_NAME: &str = "video_licensing";
const VIDEO_LICENSING_STATUS_FIELD_NAME: &str = "video_licensing_status";

static NULLABLE: &str = "__NULL__";

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[async_trait]
pub trait AssetRepository {
    async fn add(&self, asset: &Asset, user_id: &String) -> ResultE<Uuid>;
    async fn update(&self, ass: &Asset) -> ResultE<()>;
    async fn get_by_id(&self, id: &Uuid) -> ResultE<Asset>;
    async fn get_father(&self, son_id: &Uuid) -> ResultE<Option<Uuid>>;
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
    //async fn _get_by_id(&self, id: &Uuid) -> ResultE<HashMap<String, AttributeValue>> {
    async fn _get_by_id(
        &self,
        id: &Uuid,
    ) -> Result<HashMap<String, AttributeValue>, Box<dyn std::error::Error + Sync + Send>> {
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


    fn new_or_update(&self, asset: &Asset ) -> ResultE<aws_sdk_dynamodb::model::put::Builder> {

        let asset_id_av = AttributeValue::S(asset.id().to_string());
        let url_av = AttributeValue::S(asset.url().clone().unwrap().to_string());
        let creation_time_av = AttributeValue::S(iso8601(asset.creation_time()));
        let update_time_av = AttributeValue::S(iso8601(asset.last_update_time()));
        let status_av = AttributeValue::S(asset.state().to_string());

        let hash_av = AttributeValue::S(asset.hash().clone().unwrap().to_string());

        let mut items = Put::builder();
        items = items
            .item(ASSET_ID_FIELD_PK, asset_id_av.clone())
            .item(CREATIONTIME_FIELD_NAME, creation_time_av)
            .item(LASTUPDATETIME_FIELD_NAME, update_time_av)
            .item(URL_FIELD_NAME, url_av)
            .item(STATUS_FIELD_NAME, status_av)
            .item(HASH_FIELD_NAME, hash_av);

        //let longitude_av;
        match asset.longitude() {
            Some(value) => {
                let longitude_av = AttributeValue::S(value.to_string());
                items = items.item(LONGITUDE_FIELD_NAME, longitude_av);
            }
            None => {} // longitude_av = AttributeValue::S(NULLABLE.to_string()),
        }
        //let latitude_av;
        match asset.latitude() {
            Some(value) => {
                let latitude_av = AttributeValue::S(value.to_string());
                items = items.item(LATITUDE_FIELD_NAME, latitude_av);
            }
            None => {} //latitude_av = AttributeValue::S(NULLABLE.to_string()),
        }
        //let license_av;
        match asset.license() {
            Some(value) => {
                let license_av = AttributeValue::S(value.to_string());
                items = items.item(LICENSE_FIELD_NAME, license_av);
            }
            None => {} // license_av = AttributeValue::S(NULLABLE.to_string()),
        }
        //let shorter_av;
        match asset.shorter() {
            Some(value) => {
                let shorter_av = AttributeValue::S(value.to_string());
                items = items.item(SHORTER_FIELD_NAME, shorter_av);
            }
            None => {} //shorter_av = AttributeValue::S(NULLABLE.to_string()),
        }
        //let counter_av;
        match asset.counter() {
            Some(value) => {
                let counter_av = AttributeValue::N(value.to_string());
                items = items.item(COUNTER_FIELD_NAME, counter_av);
            }
            None => {} // counter_av = AttributeValue::N(NULLABLE.to_string()),
        }
        //let video_licensing_error_av;
        match asset.video_licensing_error() {
            Some(value) => {
                let video_licensing_error_av = AttributeValue::S(value.to_string());
                items = items.item( VIDEO_LICENSING_FIELD_NAME , video_licensing_error_av );
            },
            None => {},
        }

        //let video_licensing_status_av = AttributeValue::S(asset.video_licensing_status().to_string());
        items = items.item(
            VIDEO_LICENSING_STATUS_FIELD_NAME,
            AttributeValue::S(asset.video_licensing_status().to_string()),
        );
        items = items.item(
            MINTED_STATUS_FIELD_NAME,
            AttributeValue::S(asset.mint_status().to_string()),
        );
        Ok(items)
    }


}

#[async_trait]
impl AssetRepository for AssetRepo {
    async fn add(&self, asset: &Asset, user_id: &String) -> ResultE<Uuid> {
        

        let asset_id_av = AttributeValue::S(asset.id().to_string());
        let user_id_av = AttributeValue::S(user_id.clone());
        
        let items = self.new_or_update(asset).unwrap();

        let mut request = self
            .client
            .transact_write_items()
            .transact_items(
                TransactWriteItem::builder()
                    .put(items.table_name(ASSETS_TABLE_NAME).build())
                    .build(),
            )
            .transact_items(
                TransactWriteItem::builder()
                    .put(
                        Put::builder()
                            .item(OWNER_USER_ID_FIELD_PK, user_id_av.clone())
                            .item(OWNER_ASSET_ID_FIELD_PK, asset_id_av.clone())
                            .table_name(OWNERS_TABLE_NAME)
                            .build(),
                    )
                    .build(),
            );

        match asset.father() {
            None => {}
            Some(value) => {
                let father_id_av = AttributeValue::S(value.to_string());
                request = request.transact_items(
                    TransactWriteItem::builder()
                        .put(
                            Put::builder()
                                .item(ASSET_TREE_FATHER_ID_FIELD, father_id_av)
                                .item(ASSET_TREE_SON_ID_FIELD_PK, asset_id_av)
                                .table_name(ASSET_TREE_TABLE_NAME)
                                .build(),
                        )
                        .build(),
                );
            }
        }

        match request.send().await {
            Ok(_stored) => {
                let mssag = format!(
                    "Stored new item [{}] - user id: {} asset id: {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    user_id,
                    asset.id().to_string()
                );
                tracing::debug!(mssag);

                return Ok(asset.id().clone());
            }
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

    //without fathers!
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

    async fn get_by_id(
        &self,
        id: &Uuid,
    ) -> std::result::Result<Asset, Box<dyn std::error::Error + Sync + Send>> {
        let res = self._get_by_id(id).await?;
        let mut asset = Asset::new();
        mapping_from_doc_to_asset(&res, &mut asset);
        match self.get_father(id).await? {
            None => {}
            Some(val) => asset.set_father(&Some(val)),
        }
        Ok(asset)
    }

    async fn update(&self, asset: &Asset) -> ResultE<()> {

        
        let items = self.new_or_update(asset).unwrap();

        let request = self
            .client
            .transact_write_items()
            .transact_items(
                TransactWriteItem::builder()
                    .put(items.table_name(ASSETS_TABLE_NAME).build())
                    .build(),
            );

/* 

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
        let minted_status_av = AttributeValue::S(asset.mint_status().to_string());

        let shorter_av;
        match asset.shorter() {
            Some(value) => shorter_av = AttributeValue::S(value.to_string()),
            None => shorter_av = AttributeValue::S(NULLABLE.to_string()),
        }
        let counter_av;
        match asset.counter() {
            Some(value) => counter_av = AttributeValue::N(value.to_string()),
            None => counter_av = AttributeValue::S(NULLABLE.to_string()),
        }

        let video_licensing_status_av =
            AttributeValue::S(asset.video_licensing_status().to_string());

        let video_licensing_error_av;
        match asset.video_licensing_error() {
            Some(value) => video_licensing_error_av = AttributeValue::S(value.to_string()),
            None => video_licensing_error_av = AttributeValue::S(NULLABLE.to_string()),
        }
        */
/* 
        let mut update_express = "set ".to_string();
        update_express.push_str(format!("{0} = :_url, ", URL_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :lastup, ", LASTUPDATETIME_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :_status, ", STATUS_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :_hash, ", HASH_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :longitude, ", LONGITUDE_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :latitude, ", LATITUDE_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :license, ", LICENSE_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :minted_tx, ", MINTED_FIELD_NAME).as_str());
        update_express
            .push_str(format!("{0} = :minted_status, ", MINTED_STATUS_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :shorter, ", SHORTER_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :counter, ", COUNTER_FIELD_NAME).as_str());
        update_express
            .push_str(format!("{0} = :lic_status, ", VIDEO_LICENSING_STATUS_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :lic_error", VIDEO_LICENSING_FIELD_NAME).as_str());

        let request = self
            .client
            .update_item()
            .table_name(ASSETS_TABLE_NAME)
            .key(ASSET_ID_FIELD_PK, id_av)
            .update_expression(update_express)
            .expression_attribute_values(":_url", url_av)
            .expression_attribute_values(":lastup", last_update_time_av)
            .expression_attribute_values(":_hash", hash_av)
            .expression_attribute_values(":longitude", longitude_av)
            .expression_attribute_values(":latitude", latitude_av)
            .expression_attribute_values(":license", license_av)
            .expression_attribute_values(":minted_tx", minted_tx_av)
            .expression_attribute_values(":minted_status", minted_status_av)
            .expression_attribute_values(":_status", status_av)
            .expression_attribute_values(":shorter", shorter_av)
            .expression_attribute_values(":counter", counter_av)
            .expression_attribute_values(":lic_status", video_licensing_status_av)
            .expression_attribute_values(":lic_error", video_licensing_error_av);
*/
        match request.send().await {
            Ok(_updated) => {
                let mssag = format!(
                    "Record updated at [{}] - item id: {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    asset.id().to_string()
                );
                tracing::debug!(mssag);

                Ok(())
            }
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e.to_string()
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

        let mut filter2 = HashMap::new();
        filter2.insert(OWNER_USER_ID_FIELD_PK.to_string(), user_id_av);
        filter2.insert(OWNER_ASSET_ID_FIELD_PK.to_string(), asset_id_av);

        let request = self
            .client
            //.query()
            .get_item()
            .table_name(OWNERS_TABLE_NAME)
            .set_key(Some(filter2));

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

    async fn get_father(&self, son_id: &Uuid) -> ResultE<Option<Uuid>> {
        let asset_son_id_av = AttributeValue::S(son_id.to_string());

        let mut filter = HashMap::new();
        filter.insert(ASSET_TREE_SON_ID_FIELD_PK.to_string(), asset_son_id_av);

        let request = self
            .client
            //.query()
            .get_item()
            .table_name(ASSET_TREE_TABLE_NAME)
            .set_key(Some(filter));

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
            None => {
                //return Err(FatherNoExistsError(son_id.to_string()).into());
                Ok(None)
            }
            Some(aux) => {
                let _id = aux.get(ASSET_TREE_FATHER_ID_FIELD).unwrap();
                let asset_id = _id.as_s().unwrap();
                let father_uuid = Uuid::from_str(asset_id).unwrap();
                Ok(Some(father_uuid))
                //let res = self._get_by_id(&father_uuid).await?;
                //let mut asset = Asset::new();
                //mapping_from_doc_to_asset(&res, &mut asset);
                //Ok(Some(asset))
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

    let tx_minted = doc.get(MINTED_FIELD_NAME);
    match tx_minted {
        None => asset.set_minted_tx(&None),
        Some(lati) => {
            let val = lati.as_s().unwrap();
            if val == NULLABLE {
                asset.set_minted_tx(&None)
            } else {
                let hash = H256::from_str(val).unwrap();
                asset.set_minted_tx(&Some(hash.clone()))
            }
        }
    }

    let minted_status = doc.get(MINTED_STATUS_FIELD_NAME).unwrap().as_s().unwrap();
    asset.set_minted_status(MintingStatus::from_str(minted_status).unwrap());

    let status_t = doc.get(STATUS_FIELD_NAME).unwrap().as_s().unwrap();
    let aux = AssetStatus::from_str(status_t).unwrap();
    asset.set_state(&aux);

    let video_licensing_status = doc
        .get(VIDEO_LICENSING_STATUS_FIELD_NAME)
        .unwrap()
        .as_s()
        .unwrap();
    asset.set_video_licensing_status(
        VideoLicensingStatus::from_str(video_licensing_status).unwrap(),
    );

    let video_licensing_error = doc.get(VIDEO_LICENSING_FIELD_NAME);
    match video_licensing_error {
        None => asset.set_video_licensing_error(&None),
        Some(lati) => {
            let val = lati.as_s().unwrap();
            if val == NULLABLE {
                asset.set_video_licensing_error(&None)
            } else {
                asset.set_video_licensing_error(&Some(val.clone()))
            }
        }
    }

    let shorter = doc.get(SHORTER_FIELD_NAME);
    match shorter {
        None => asset.set_shorter(&None),
        Some(lati) => {
            let val = lati.as_s().unwrap();
            if val == NULLABLE {
                asset.set_shorter(&None)
            } else {
                asset.set_shorter(&Some(val.clone()))
            }
        }
    }

    let counter = doc.get(COUNTER_FIELD_NAME);
    match counter {
        None => asset.set_counter(&None),
        Some(lati) => {
            let val = lati.as_n().unwrap();
            if val == NULLABLE {
                asset.set_counter(&None)
            } else {
                let num_op = u64::from_str_radix(val.as_str(), 10);
                match num_op {
                    Err(e) => {
                        error!("counter parser error! {}", val);
                        error!("{}", e);
                        asset.set_counter(&None)
                    }
                    Ok(num) => asset.set_counter(&Some(num.clone())),
                }
            }
        }
    }
    /*let _x_ = doc.get(   );
    match _x_ {
        None => asset.set_(&None),
        Some(lati) => {
            let val = lati.as_s().unwrap();
            if val == NULLABLE {
                asset.set_(&None)
            } else {
                asset.set_(&Some(val.clone()))
            }
        }
    }*/
}
