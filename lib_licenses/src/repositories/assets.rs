use std::collections::HashMap;
use std::str::FromStr;

use tracing::{error, info};
use url::Url;
use uuid::Uuid;
use web3::types::H256;

use crate::errors::asset::{AssetDynamoDBError, AssetNoExistsError, AssetTreeError};
use crate::errors::owner::{OwnerDynamoDBError, OwnerNoExistsError};
use crate::models::asset::{Asset, AssetStatus, MintingStatus, VideoLicensingStatus, SourceType};
use crate::models::owner::Owner;
use crate::models::video::VideoProcessStatus;
use async_trait::async_trait;
use aws_sdk_dynamodb::model::{AttributeValue, Put, Select, TransactWriteItem};
use aws_sdk_dynamodb::Client;
use chrono::{
    prelude::{DateTime, Utc},
    Local,
};
use lib_config::config::Config;

use super::owners::mapping_from_doc_to_owner;
use super::schema_asset::{
    ASSETS_TABLE_NAME, ASSET_ID_FIELD_PK, ASSET_TREE_FATHER_ID_FIELD_PK, ASSET_TREE_FATHER_INDEX,
    ASSET_TREE_SON_ID_FIELD_PK, ASSET_TREE_TABLE_NAME,
    URL_FIELD_NAME, URL_INDEX_NAME,
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
const VIDEO_PROCESS_STATUS_FIELD_NAME: &str = "video_processing_status";

const SOURCE_FIELD_NAME: &str = "source";
const SOURCE_DETAILS_FIELD_NAME: &str = "source_details";

static NULLABLE: &str = "__NULL__";

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[async_trait]
pub trait AssetRepository {
    async fn add(&self, asset: &Asset, user_id: &String) -> ResultE<Uuid>;
    async fn update(&self, ass: &Asset) -> ResultE<()>;
    async fn get_by_id(&self, id: &Uuid) -> ResultE<Asset>;
    async fn get_by_url(&self, url: &Url) -> ResultE<Asset>;
    async fn get_father(&self, son_id: &Uuid) -> ResultE<Option<Uuid>>;
    async fn get_sons(&self, id: &Uuid) -> ResultE<Vec<Uuid>>;
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

    fn new_or_update(&self, asset: &Asset) -> ResultE<aws_sdk_dynamodb::model::put::Builder> {
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
                items = items.item(VIDEO_LICENSING_FIELD_NAME, video_licensing_error_av);
            }
            None => {}
        }
        //let minted_tx_av;
        match asset.minted_tx() {
            Some(value) => {
                let minted_tx_av = AttributeValue::S(format!("{:?}", value));
                items = items.item(MINTED_FIELD_NAME, minted_tx_av);
            }
            None => {}
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

        match asset.video_process_status() {
            Some(value) => {
                let video_process_status_av = AttributeValue::S(value.to_string());
                items = items.item(VIDEO_PROCESS_STATUS_FIELD_NAME, video_process_status_av);
            }
            None => {}
        }
        match asset.source() {
            Some(value) => {
                let source_av = AttributeValue::S(value.to_string());
                items = items.item( SOURCE_FIELD_NAME , source_av);
            }
            None => {}
        }
        match asset.source_details() {
            Some(value) => {
                let source_det_av = AttributeValue::S(value.to_string());
                items = items.item( SOURCE_DETAILS_FIELD_NAME , source_det_av);
            }
            None => {}
        }
        Ok(items)
    }
}

#[async_trait]
impl AssetRepository for AssetRepo {
    async fn add(&self, asset: &Asset, user_id: &String) -> ResultE<Uuid> {
        let asset_id_av = AttributeValue::S(asset.id().to_string());
        let user_id_av = AttributeValue::S(user_id.clone());

        let items = self.new_or_update(asset).unwrap();

        info!("common data ready");
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

        info!("owners added");
        match asset.father() {
            None => {}
            Some(value) => {
                let father_id_av = AttributeValue::S(value.to_string());
                request = request.transact_items(
                    TransactWriteItem::builder()
                        .put(
                            Put::builder()
                                .item(ASSET_TREE_FATHER_ID_FIELD_PK, father_id_av)
                                .item(ASSET_TREE_SON_ID_FIELD_PK, asset_id_av)
                                .table_name(ASSET_TREE_TABLE_NAME)
                                .build(),
                        )
                        .build(),
                );
            }
        }

        info!("sending request to dynamodb");
        match request.send().await {
            Ok(_stored) => {
                let mssag = format!(
                    "Stored new item [{}] - user id: {} asset id: {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    user_id,
                    asset.id().to_string()
                );
                info!(mssag);

                return Ok(asset.id().clone());
            }
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                error!(mssag);
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
                        match self.get_father(asset.id()).await? {
                            None => {}
                            Some(val) => asset.set_father(&Some(val)),
                        }

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

    async fn get_by_url(&self, url: &Url) -> ResultE<Asset> {
        let asset_url_av = AttributeValue::S(url.to_string());

        let mut filter = "".to_string();
        filter.push_str(URL_FIELD_NAME);
        filter.push_str(" = :value");

        let request = self
            .client
            .query()
            .table_name(ASSETS_TABLE_NAME)
            .index_name(URL_INDEX_NAME)
            .key_condition_expression(filter)
            .expression_attribute_values(":value".to_string(), asset_url_av)
            .select(Select::AllProjectedAttributes);
        //.key(OWNER_USER_ID_FIELD_PK, _id_av.clone());

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(AssetTreeError(e.to_string()).into());
            }
            Ok(data) => {
                let op_items = data.items();
                match op_items {
                    None => {
                        return Err(AssetNoExistsError("url doesn't exist".to_string()).into());
                    }
                    Some(aux) => {
                        if aux.len() == 0 {
                            return Err(AssetNoExistsError("url doesn't exist".to_string()).into());
                        } else {
                            let doc = aux[0].clone();
                            let ass1_id = doc.get(ASSET_ID_FIELD_PK).unwrap();
                            let ass1_id1 = ass1_id.as_s().unwrap();
                            let ass1_id1_1 = Uuid::from_str(ass1_id1).unwrap();
                            let res = self.get_by_id(&ass1_id1_1).await?;
                            Ok(res)
                        }
                    }
                }
            }
        }
    }

    

    async fn update(&self, asset: &Asset) -> ResultE<()> {
        let items = self.new_or_update(asset).unwrap();

        let request = self.client.transact_write_items().transact_items(
            TransactWriteItem::builder()
                .put(items.table_name(ASSETS_TABLE_NAME).build())
                .build(),
        );

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

    async fn get_sons(&self, id: &Uuid) -> ResultE<Vec<Uuid>> {
        let mut queried = Vec::new();
        let asset_father_id_av = AttributeValue::S(id.to_string());

        let mut filter = "".to_string();
        filter.push_str(ASSET_TREE_FATHER_ID_FIELD_PK);
        filter.push_str(" = :value");

        let request = self
            .client
            .query()
            .table_name(ASSET_TREE_TABLE_NAME)
            .index_name(ASSET_TREE_FATHER_INDEX)
            .key_condition_expression(filter)
            .expression_attribute_values(":value".to_string(), asset_father_id_av)
            .select(Select::AllProjectedAttributes);
        //.key(OWNER_USER_ID_FIELD_PK, _id_av.clone());

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(AssetTreeError(e.to_string()).into());
            }
            Ok(data) => {
                let op_items = data.items();
                match op_items {
                    None => {
                        return Err(AssetTreeError("id doesn't exist".to_string()).into());
                    }
                    Some(aux) => {
                        for doc in aux {
                            let ass1_id = doc.get(ASSET_TREE_SON_ID_FIELD_PK).unwrap();
                            let ass1_id1 = ass1_id.as_s().unwrap();
                            let ass1_id1_1 = Uuid::from_str(ass1_id1).unwrap();
                            queried.push(ass1_id1_1.clone());
                        }
                    }
                }
            }
        }

        Ok(queried)
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
            //let res = self._get_by_id(ass.asset_id()).await?;
            //let mut asset = Asset::new();
            //mapping_from_doc_to_asset(&res, &mut asset);

            let asset = self.get_by_id(ass.asset_id()).await?;

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
        //let res = self._get_by_id(own.asset_id()).await?;
        //let mut asset = Asset::new();
        //mapping_from_doc_to_asset(&res, &mut asset);
        let asset = self.get_by_id(own.asset_id()).await?;
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
                let _id = aux.get(ASSET_TREE_FATHER_ID_FIELD_PK).unwrap();
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

    let video_proc_sts = doc.get(VIDEO_PROCESS_STATUS_FIELD_NAME);
    match video_proc_sts {
        None => asset.set_video_process_status(&None),
        Some(vid_st) => {
            let val = vid_st.as_s().unwrap();
            if val == NULLABLE {
                asset.set_video_process_status(&None)
            } else {
                let st_op = VideoProcessStatus::from_str(val);
                match st_op {
                    Err(e) => {
                        error!("video process status parser error! {}", val);
                        error!("{}", e);
                        asset.set_video_process_status(&None)
                    }
                    Ok(state) => asset.set_video_process_status(&Some(state)),
                }
            }
        }
    }
let source = doc.get( SOURCE_FIELD_NAME   );
    match source{
        None => asset.set_source(&None),
        Some(value) => {
            let val = value.as_s().unwrap();
            if val == NULLABLE {
                asset.set_source(&None)
            } else {
                let st_op = SourceType::from_str(val);
                match st_op {
                    Err(e) => {
                        error!("source type parser error! {}", val);
                        error!("{}", e);
                        asset.set_source(&None)
                    }
                    Ok(state) => asset.set_source(&Some(state)),
                }
            }
        }
    }

let source_details = doc.get(SOURCE_DETAILS_FIELD_NAME   );
    match source_details {
        None => asset.set_source_details(&None),
        Some(lati) => {
            let val = lati.as_s().unwrap();
            if val == NULLABLE {
                asset.set_source_details(&None)
            } else {
                asset.set_source_details(&Some(val.clone()))
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
