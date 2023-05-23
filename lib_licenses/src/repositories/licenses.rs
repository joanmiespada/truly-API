use std::collections::HashMap;
use std::str::FromStr;

use aws_sdk_dynamodb::types::Select;
use uuid::Uuid;

use crate::errors::license::{LicenseCreationError, LicenseNotFoundError};
use crate::models::license::{License, LicenseStatus, Royalty};
use async_trait::async_trait;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use chrono::{
    prelude::{DateTime, Utc},
    Local,
};
use lib_config::config::Config;

use super::schema_licenses::{
    LICENSES_ASSET_ID_INDEX, LICENSES_TABLE_NAME, LICENSE_ASSET_ID_FIELD_PK, LICENSE_ID_FIELD_PK,
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
    async fn get_by_id(&self, id: &Uuid) -> ResultE<License>;
    async fn get_by_asset_id(&self, asset_id: &Uuid) -> ResultE<Vec<License>>;
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
}

#[async_trait]
impl LicenseRepository for LicenseRepo {
    async fn create(&self, license: &mut License) -> ResultE<()> {
        let id_av = AttributeValue::S(license.id().to_string());
        let creation_time_av = AttributeValue::S(iso8601(*license.creation_time()));
        let last_update_time_av = AttributeValue::S(iso8601(*license.last_update_time()));
        let asset_id_av = AttributeValue::S(license.asset_id().to_string());
        let version_av = AttributeValue::N(license.version().to_string());

        let request = self
            .client
            .put_item()
            .table_name(LICENSES_TABLE_NAME)
            .item(LICENSE_ID_FIELD_PK, id_av)
            .item(CREATION_TIME_FIELD_NAME, creation_time_av)
            .item(LAST_UPDATE_TIME_FIELD_NAME, last_update_time_av)
            .item(LICENSE_ASSET_ID_FIELD_PK, asset_id_av)
            .item(LICENSE_VERSION_FIELD, version_av);

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

    async fn get_by_id(&self, id: &Uuid) -> ResultE<License> {
        let id_av = AttributeValue::S(id.to_string());

        let mut filter = "".to_string();
        filter.push_str(LICENSE_ID_FIELD_PK);
        filter.push_str(" = :value");

        let res = self
            .get_by_filter(&filter, &":value".to_string(), "", id_av)
            .await?;

        if res.is_empty() {
            Err(LicenseNotFoundError("License not found".to_string()).into())
        } else {
            let license = res.into_iter().next().unwrap();
            Ok(license)
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
