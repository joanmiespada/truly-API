use aws_sdk_dynamodb::types::Select;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

use crate::errors::license::{LicenseCreationError, LicenseDynamoDBError, LicenseNotFoundError};
use crate::models::license::{License, LicenseStatus, Royalty};
use async_trait::async_trait;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use chrono::{
    prelude::{DateTime, Utc},
    Local,
};
use lib_config::config::Config;

use super::schema_licenses::{
    LICENSES_ASSET_ID_INDEX, LICENSES_TABLE_NAME, LICENSE_ASSET_ID_FIELD_PK, LICENSE_ID_FIELD_PK,LICENSES_LICENSE_ID_INDEX
};
pub const CREATION_TIME_FIELD_NAME: &str = "creationTime";
pub const LAST_UPDATE_TIME_FIELD_NAME: &str = "lastUpdateTime";
pub const LICENSE_VERSION_FIELD: &str = "version";
pub const RIGHT_TO_FREE_DISTRIBUTE_FIELD: &str = "rightToFreeDistribute";
pub const IF_YOU_DISTRIBUTE_MENTION_ME_FIELD: &str = "ifYouDistributeMentionMe";
pub const RIGHT_TO_MODIFY_FIELD: &str = "rightToModify";
pub const IF_YOU_MODIFY_MENTION_ME_FIELD: &str = "ifYouModifyMentionMe";
pub const RIGHT_TO_USE_BROADCAST_MEDIA_FIELD: &str = "rightToUseBroadcastMedia";
pub const RIGHT_TO_USE_PRESS_MEDIA_FIELD: &str = "rightToUsePressMedia";
pub const LICENSE_STATUS_FIELD: &str = "status";
pub const ROYALTIES_FIELD: &str = "royalties";

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[async_trait]
pub trait LicenseRepository {
    async fn create(&self, license: &mut License) -> ResultE<()>;
    async fn get_by_id(&self, license_id: &Uuid, asset_id: &Uuid) -> ResultE<Option<License>>;
    async fn get_by_license_id(&self, license_id: &Uuid) -> ResultE<Option<License>>;
    async fn get_by_asset_id(&self, asset_id: &Uuid) -> ResultE<Vec<License>>;
    async fn get_all(&self, _page_number: u32, _page_size: u32) -> ResultE<Vec<License>>;
    async fn update(&self, license: &License) -> ResultE<()>;
    async fn delete(&self, id: &Uuid) -> ResultE<()>;
}

#[derive(Clone, Debug)]
pub struct LicenseRepo {
    client: Client,
}

impl LicenseRepo {
    pub fn new(conf: &Config) -> LicenseRepo {
        LicenseRepo {
            client: Client::new(conf.aws_config()),
        }
    }
    async fn get_by_filter(
        &self,
        filter: &String,
        label: &String,
        index_name: &str,
        av: AttributeValue,
    ) -> ResultE<Vec<License>> {
        let mut queried = Vec::new();

        let request = self
            .client
            .query()
            .table_name(LICENSES_TABLE_NAME)
            .index_name(index_name)
            .key_condition_expression(filter)
            .expression_attribute_values(label, av)
            .select(Select::AllProjectedAttributes);

        let results = request.send().await;
        match results {
            Err(e) => {
                let message = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(message);
                return Err(LicenseCreationError(e.to_string()).into());
            }
            Ok(data) => {
                let op_items = data.items();
                match op_items {
                    None => {
                        return Err(LicenseNotFoundError("License not found".to_string()).into());
                    }
                    Some(aux) => {
                        for doc in aux {
                            let mut license = License::new();
                            mapping_from_doc_to_license(doc, &mut license);
                            queried.push(license.clone());
                        }
                    }
                }
            }
        }

        Ok(queried)
    }

    async fn _get_by_id(
        &self,
        license_id: &Uuid,
        asset_id: &Uuid,
    ) -> Result<HashMap<String, AttributeValue>, Box<dyn std::error::Error + Sync + Send>> {
        let asset_id_av = AttributeValue::S(asset_id.to_string());
        let license_id_av = AttributeValue::S(license_id.to_string());

        let request = self
            .client
            .get_item()
            .table_name(LICENSES_TABLE_NAME)
            .key(LICENSE_ID_FIELD_PK, license_id_av)
            .key(LICENSE_ASSET_ID_FIELD_PK, asset_id_av);

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(LicenseDynamoDBError(e.to_string()).into());
            }
            Ok(res) => match res.item {
                None => Err(LicenseNotFoundError("id doesn't exist".to_string()).into()),
                Some(aux) => Ok(aux),
            },
        }
    }
}

#[async_trait]
impl LicenseRepository for LicenseRepo {
    async fn create(&self, license: &mut License) -> ResultE<()> {
        let id_av = AttributeValue::S(license.id().to_string());
        let creation_time_av = AttributeValue::S(license.creation_time().to_rfc3339());
        let last_update_time_av = AttributeValue::S(license.last_update_time().to_rfc3339());
        let asset_id_av = AttributeValue::S(license.asset_id().to_string());
        let version_av = AttributeValue::N(license.version().to_string());
        let right_to_free_distribute_av = AttributeValue::Bool(license.right_to_free_distribute());
        let if_you_distribute_mention_me_av =
            AttributeValue::Bool(license.if_you_distribute_mention_me());
        let right_to_modify_av = AttributeValue::Bool(license.right_to_modify());
        let if_you_modify_mention_me_av = AttributeValue::Bool(license.if_you_modify_mention_me());
        let right_to_use_broadcast_media_av =
            AttributeValue::Bool(license.right_to_use_broadcast_media());
        let right_to_use_press_media_av = AttributeValue::Bool(license.right_to_use_press_media());
        let status_av = AttributeValue::S(license.status().to_string());

        let mut rights_av = Vec::new();
        for royalty in license.rights() {
            let royalty_av = AttributeValue::M(
                maplit::hashmap! {
                    "price".to_string() => AttributeValue::N(royalty.price.to_string()),
                    "location".to_string() => AttributeValue::S(royalty.location.to_string()),
                }
                .into(),
            );
            rights_av.push(royalty_av);
        }

        let request = self
            .client
            .put_item()
            .table_name(LICENSES_TABLE_NAME)
            .item(LICENSE_ID_FIELD_PK, id_av)
            .item(CREATION_TIME_FIELD_NAME, creation_time_av)
            .item(LAST_UPDATE_TIME_FIELD_NAME, last_update_time_av)
            .item(LICENSE_ASSET_ID_FIELD_PK, asset_id_av)
            .item(LICENSE_VERSION_FIELD, version_av)
            .item(RIGHT_TO_FREE_DISTRIBUTE_FIELD, right_to_free_distribute_av)
            .item(
                IF_YOU_DISTRIBUTE_MENTION_ME_FIELD,
                if_you_distribute_mention_me_av,
            )
            .item(RIGHT_TO_MODIFY_FIELD, right_to_modify_av)
            .item(IF_YOU_MODIFY_MENTION_ME_FIELD, if_you_modify_mention_me_av)
            .item(
                RIGHT_TO_USE_BROADCAST_MEDIA_FIELD,
                right_to_use_broadcast_media_av,
            )
            .item(RIGHT_TO_USE_PRESS_MEDIA_FIELD, right_to_use_press_media_av)
            .item(LICENSE_STATUS_FIELD, status_av)
            .item(ROYALTIES_FIELD, AttributeValue::L(rights_av));

        match request.send().await {
            Ok(_) => Ok(()),
            Err(e) => {
                let message = format!("Error creating license: {}", e);
                tracing::error!("{}", message);
                Err(LicenseDynamoDBError(message).into())
            }
        }
    }

    async fn get_by_id(&self, license_id: &Uuid, asset_id: &Uuid) -> ResultE<Option<License>> {
        let res = self._get_by_id(license_id, asset_id).await;
        match res {
            Err(_) => Ok(None),
            Ok(doc) => {
                let mut license = License::new();
                mapping_from_doc_to_license(&doc, &mut license);
                Ok(Some(license))
            }
        }
    }

    async fn get_by_license_id(&self, license_id: &Uuid) -> ResultE<Option<License>> {
        let license_id_av = AttributeValue::S(license_id.to_string());

        let mut filter = "".to_string();
        filter.push_str( LICENSE_ID_FIELD_PK);
        filter.push_str(" = :value");

        let res = self
            .get_by_filter(
                &filter,
                &":value".to_string(),
               LICENSES_LICENSE_ID_INDEX,
                license_id_av,
            )
            .await;
        match res {
            Err(e) => {
                Err(e)
                //error!("{}", e);
                //return Err(LicenseDynamoDBError(e.to_string()).into());
            }
            Ok(res) => {
                if res.len() == 0 {
                    return Ok(None);
                } else {
                    return Ok(Some(res[0].clone()));
                }
            }
        }
    }
    async fn get_by_asset_id(&self, asset_id: &Uuid) -> ResultE<Vec<License>> {
        let asset_id_av = AttributeValue::S(asset_id.to_string());

        let mut filter = "".to_string();
        filter.push_str(LICENSE_ASSET_ID_FIELD_PK);
        filter.push_str(" = :value");

        let res = self
            .get_by_filter(
                &filter,
                &":value".to_string(),
                LICENSES_ASSET_ID_INDEX,
                asset_id_av,
            )
            .await?;

        Ok(res)
    }

    async fn get_all(&self, _page_number: u32, _page_size: u32) -> ResultE<Vec<License>> {
        let mut queried = Vec::new();

        let results = self
            .client
            .scan()
            .table_name(LICENSES_TABLE_NAME)
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
                return Err(LicenseDynamoDBError(e.to_string()).into());
            }
            Ok(result) => {
                if let Some(docs) = result.items {
                    for doc in docs {
                        let mut lic = License::new();

                        mapping_from_doc_to_license(&doc, &mut lic);

                        queried.push(lic.clone());
                    }
                }
            }
        }

        Ok(queried)
    }

    async fn update(&self, license: &License) -> ResultE<()> {
        let last_update_time_av = AttributeValue::S(iso8601(*license.last_update_time()));

        let request = self
            .client
            .update_item()
            .table_name(LICENSES_TABLE_NAME)
            .key(
                LICENSE_ID_FIELD_PK,
                AttributeValue::S(license.id().to_string()),
            )
            .update_expression(format!("SET {} = :value", LAST_UPDATE_TIME_FIELD_NAME))
            .expression_attribute_values(":value", last_update_time_av);

        match request.send().await {
            Ok(_) => Ok(()),
            Err(e) => {
                let message = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(message);
                return Err(LicenseCreationError(e.to_string()).into());
            }
        }
    }

    async fn delete(&self, id: &Uuid) -> ResultE<()> {
        let id_av = AttributeValue::S(id.to_string());

        let request = self
            .client
            .delete_item()
            .table_name(LICENSES_TABLE_NAME)
            .key(LICENSE_ID_FIELD_PK, id_av);

        match request.send().await {
            Ok(_) => Ok(()),
            Err(e) => {
                let message = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(message);
                return Err(LicenseCreationError(e.to_string()).into());
            }
        }
    }
}

fn iso8601(st: DateTime<Utc>) -> String {
    let dt: DateTime<Utc> = st.into();
    format!("{}", dt.format("%+"))
}

fn from_iso8601(st: String) -> DateTime<Utc> {
    let aux = st.parse::<DateTime<Utc>>().unwrap();
    aux
}

fn mapping_from_attr_to_royalty(attr: &AttributeValue) -> Option<Royalty> {
    if let Ok(m) = attr.as_m() {
        let price = m
            .get("price")
            .unwrap()
            .as_n()
            .unwrap()
            .parse::<f32>()
            .ok()?;

        let location = m.get("location").unwrap().as_s().unwrap().clone();

        Some(Royalty { price, location })
    } else {
        None
    }
}

fn mapping_from_doc_to_license(doc: &HashMap<String, AttributeValue>, license: &mut License) {
    if let Some(id_attr) = doc.get(LICENSE_ID_FIELD_PK) {
        if let Ok(id) = id_attr.as_s() {
            if let Ok(uuid) = Uuid::parse_str(id) {
                license.set_id(uuid);
            }
        }
    }

    if let Some(creation_time_attr) = doc.get(CREATION_TIME_FIELD_NAME) {
        if let Ok(creation_time) = creation_time_attr.as_s().as_ref() {
            let dt = from_iso8601(creation_time.to_string());
            license.set_creation_time(dt);
        }
    }

    if let Some(last_update_time_attr) = doc.get(LAST_UPDATE_TIME_FIELD_NAME) {
        if let Ok(last_update_time) = last_update_time_attr.as_s().as_ref() {
            let dt = from_iso8601(last_update_time.to_string());
            license.set_last_update_time(dt);
        }
    }

    if let Some(asset_id_attr) = doc.get(LICENSE_ASSET_ID_FIELD_PK) {
        if let Ok(asset_id) = asset_id_attr.as_s().as_ref() {
            if let Ok(uuid) = Uuid::parse_str(asset_id) {
                license.set_asset_id(uuid);
            }
        }
    }

    if let Some(version_attr) = doc.get(LICENSE_VERSION_FIELD) {
        if let Ok(version) = version_attr.as_n().as_ref() {
            if let Ok(v) = version.parse::<u8>() {
                license.set_version(v);
            }
        }
    }

    if let Some(status_attr) = doc.get(LICENSE_STATUS_FIELD) {
        if let Ok(status) = status_attr.as_s().as_ref() {
            if let Ok(license_status) = LicenseStatus::from_str(status) {
                license.set_status(license_status);
            }
        }
    }

    if let Some(right_to_free_distribute_attr) = doc.get(RIGHT_TO_FREE_DISTRIBUTE_FIELD) {
        if let Ok(right_to_free_distribute) = right_to_free_distribute_attr.as_bool() {
            license.set_right_to_free_distribute(*right_to_free_distribute);
        }
    }

    if let Some(if_you_distribute_mention_me_attr) = doc.get(IF_YOU_DISTRIBUTE_MENTION_ME_FIELD) {
        if let Ok(if_you_distribute_mention_me) = if_you_distribute_mention_me_attr.as_bool() {
            license.set_if_you_distribute_mention_me(*if_you_distribute_mention_me);
        }
    }

    if let Some(right_to_modify_attr) = doc.get(RIGHT_TO_MODIFY_FIELD) {
        if let Ok(right_to_modify) = right_to_modify_attr.as_bool() {
            license.set_right_to_modify(*right_to_modify);
        }
    }

    if let Some(if_you_modify_mention_me_attr) = doc.get(IF_YOU_MODIFY_MENTION_ME_FIELD) {
        if let Ok(if_you_modify_mention_me) = if_you_modify_mention_me_attr.as_bool() {
            license.set_if_you_modify_mention_me(*if_you_modify_mention_me);
        }
    }

    if let Some(right_to_use_broadcast_media_attr) = doc.get(RIGHT_TO_USE_BROADCAST_MEDIA_FIELD) {
        if let Ok(right_to_use_broadcast_media) = right_to_use_broadcast_media_attr.as_bool() {
            license.set_right_to_use_broadcast_media(*right_to_use_broadcast_media);
        }
    }

    if let Some(right_to_use_press_media_attr) = doc.get(RIGHT_TO_USE_PRESS_MEDIA_FIELD) {
        if let Ok(right_to_use_press_media) = right_to_use_press_media_attr.as_bool() {
            license.set_right_to_use_press_media(*right_to_use_press_media);
        }
    }

    if let Some(rights_attr) = doc.get(ROYALTIES_FIELD) {
        if let Ok(rights) = rights_attr.as_l() {
            let royalties = rights
                .iter()
                .filter_map(|attr| mapping_from_attr_to_royalty(attr))
                .collect();
            license.set_rights(royalties);
        }
    }
}
