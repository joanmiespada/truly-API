use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, types::{AttributeValue,Put,TransactWriteItem}};
use lib_config::config::Config;
use uuid::Uuid;
use crate::models::notification::{Notification, NotificationBuilder};
use std::{collections::HashMap, str::FromStr};
use lib_config::result::ResultE;
use chrono::{DateTime, Utc};

use super::schema_notifications::{NOTIFICATIONS_TABLE_NAME, NOTIFICATION_ID_FIELD_PK};

pub const CREATION_TIME: &str = "creation_time";
pub const SOURCE_TYPE: &str = "source_type";
pub const ORIGIN_HASH_ID: &str = "origin_hash_id";
pub const ORIGIN_HASH_TYPE: &str = "origin_hash_type";
pub const ORIGIN_FRAME_ID: &str = "origin_frame_id";
pub const ORIGIN_FRAME_SECOND: &str = "origin_frame_second";
pub const ORIGIN_FRAME_URL: &str = "origin_frame_url";
pub const ORIGIN_ASSET_ID: &str = "origin_asset_id";
pub const SIMILAR_FRAME_ID: &str = "similar_frame_id";
pub const SIMILAR_FRAME_SECOND: &str = "similar_frame_second";
pub const SIMILAR_ASSET_ID: &str = "similar_asset_id";
pub const SIMILAR_FRAME_URL: &str = "similar_frame_url";


#[async_trait]
pub trait NotificationRepository{
    async fn add(&self, notification: &Notification) -> ResultE<()>;
    async fn get(&self, notification_id: Uuid ) -> ResultE<Option<Notification>>;
    async fn update(&self, notification: &Notification) -> ResultE<()>;
    async fn delete(&self, notification_id: Uuid) -> ResultE<()>;

}


pub struct NotificationRepo {
    client: Client,
}

impl NotificationRepo {

    pub fn new(conf: &Config) -> Self {
        NotificationRepo{
            client: Client::new(conf.aws_config()),
        }
    }

    async fn add_or_update(&self, notification: &Notification) -> ResultE<()> {
        
        let id_av = AttributeValue::S(notification.id().to_string());
        let creation_time_av = AttributeValue::S(notification.creation_time().to_string());
        
        let mut items = Put::builder();
        
        items = items.item(NOTIFICATION_ID_FIELD_PK, id_av);
        items = items.item(CREATION_TIME,creation_time_av);

        if let Some(source_type) = notification.source_type() {
            let source_type_av = AttributeValue::S(source_type.to_string());
            items = items.item(SOURCE_TYPE, source_type_av);
        }

        if let Some(origin_hash_id) = notification.origin_hash_id() {
            let origin_hash_id_av = AttributeValue::S(origin_hash_id.to_string());
            items = items.item(ORIGIN_HASH_ID, origin_hash_id_av);
        }

        if let Some(origin_hash_type) = notification.origin_hash_type() {
            let origin_hash_type_av = AttributeValue::S(origin_hash_type.to_string());
            items = items.item(ORIGIN_HASH_TYPE, origin_hash_type_av);
        }
        
        if let Some(origin_frame_id) = notification.origin_frame_id() {
            let origin_frame_id_av = AttributeValue::S(origin_frame_id.to_string());
            items = items.item(ORIGIN_FRAME_ID, origin_frame_id_av);
        }

        if let Some(origin_frame_second) = notification.origin_frame_second() {
            let origin_frame_second_av = AttributeValue::S(origin_frame_second.to_string());
            items = items.item(ORIGIN_FRAME_SECOND, origin_frame_second_av);
        }

        if let Some(origin_frame_url) = notification.origin_frame_url() {
            let origin_frame_url_av = AttributeValue::S(origin_frame_url.to_string());
            items = items.item(ORIGIN_FRAME_URL, origin_frame_url_av);
        }

        if let Some(origin_asset_id) = notification.origin_asset_id() {
            let origin_asset_id_av = AttributeValue::S(origin_asset_id.to_string());
            items = items.item(ORIGIN_ASSET_ID, origin_asset_id_av);
        }

        if let Some(similar_frame_id) = notification.similar_frame_id() {
            let similar_frame_id_av = AttributeValue::S(similar_frame_id.to_string());
            items = items.item(SIMILAR_FRAME_ID, similar_frame_id_av);
        }

        if let Some(similar_frame_second) = notification.similar_frame_second() {
            let similar_frame_second_av = AttributeValue::S(similar_frame_second.to_string());
            items = items.item(SIMILAR_FRAME_SECOND, similar_frame_second_av);
        }

        if let Some(similar_asset_id) = notification.similar_asset_id() {
            let similar_asset_id_av = AttributeValue::S(similar_asset_id.to_string());
            items = items.item(SIMILAR_ASSET_ID, similar_asset_id_av);
        }

        if let Some(similar_frame_url) = notification.similar_frame_url() {
            let similar_frame_url_av = AttributeValue::S(similar_frame_url.to_string());
            items = items.item(SIMILAR_FRAME_URL, similar_frame_url_av);
        }

        let request = self.client.transact_write_items().transact_items(
            TransactWriteItem::builder()
                .put(items.table_name(NOTIFICATIONS_TABLE_NAME.clone()).build())
                .build(),
        );
        
        match request.send().await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    fn mapping_from_doc(doc: &HashMap<String, AttributeValue>, notification: &mut Notification) {
        let _id = doc.get(NOTIFICATION_ID_FIELD_PK).unwrap();
        let notif_id = _id.as_s().unwrap();
        let notif_uuid = Uuid::from_str(notif_id).unwrap();
        notification.set_id(notif_uuid);
    
        let creation_time = doc.get(CREATION_TIME).unwrap();
        notification.set_creation_time(creation_time.as_s().unwrap().parse::<DateTime<Utc>>().unwrap());

    
        if let Some(value) = doc.get(SOURCE_TYPE){
            let val = value.as_s().unwrap();
            notification.set_source_type(val.to_string());
        }

        if let Some(value) = doc.get(ORIGIN_HASH_ID){
            let val = value.as_s().unwrap();
            let uuid = Uuid::from_str(val).unwrap();
            notification.set_origin_hash_id(uuid);
        }

        if let Some(value) = doc.get(ORIGIN_HASH_TYPE){
            let val = value.as_s().unwrap();
            notification.set_origin_hash_type(val.to_string());
        }

        if let Some(value) = doc.get(ORIGIN_FRAME_ID){
            let val = value.as_s().unwrap();
            let uuid = Uuid::from_str(val).unwrap();
            notification.set_origin_frame_id(uuid);
        }

        if let Some(value) = doc.get(ORIGIN_FRAME_SECOND){
            let val = value.as_s().unwrap();
            let second = val.parse::<f64>().unwrap();
            notification.set_origin_frame_second(second);
        }

        if let Some(value) = doc.get(ORIGIN_FRAME_URL){
            let val = value.as_s().unwrap();
            notification.set_origin_frame_url(val.to_string());
        }

        if let Some(value) = doc.get(ORIGIN_ASSET_ID){
            let val = value.as_s().unwrap();
            let uuid = Uuid::from_str(val).unwrap();
            notification.set_origin_asset_id(uuid);
        }

        if let Some(value) = doc.get(SIMILAR_FRAME_ID){
            let val = value.as_s().unwrap();
            let uuid = Uuid::from_str(val).unwrap();
            notification.set_similar_frame_id(uuid);
        }

        if let Some(value) = doc.get(SIMILAR_FRAME_SECOND){
            let val = value.as_s().unwrap();
            let second = val.parse::<f64>().unwrap();
            notification.set_similar_frame_second(second);
        }

        if let Some(value) = doc.get(SIMILAR_ASSET_ID){
            let val = value.as_s().unwrap();
            let uuid = Uuid::from_str(val).unwrap();
            notification.set_similar_asset_id(uuid);
        }

        if let Some(value) = doc.get(SIMILAR_FRAME_URL){
            let val = value.as_s().unwrap();
            notification.set_similar_frame_url(val.to_string());
        }

    }


}
#[async_trait]
impl NotificationRepository for NotificationRepo {
    async fn add(&self, notification: &Notification) -> ResultE<()> {
        self.add_or_update(notification).await
    }

    async fn get(&self, notification_id: Uuid) -> ResultE<Option<Notification>> {

        let id_av = AttributeValue::S(notification_id.to_string());
        let output = self.client.get_item()
            .table_name(NOTIFICATIONS_TABLE_NAME.clone())
            .key(NOTIFICATION_ID_FIELD_PK, id_av)
            .send()
            .await?;

        if let Some(item) = output.item {
            let mut notification = NotificationBuilder::default().build();
            Self::mapping_from_doc(&item, &mut notification);
            return Ok(Some(notification));
        }
        Ok(None)
    }

    async fn update(&self, notification: &Notification) -> ResultE<()> {
        self.add_or_update(notification).await
    }

    async fn delete(&self, notification_id:Uuid) -> ResultE<()> {

        let id_av = AttributeValue::S(notification_id.to_string());
        self.client.delete_item()
            .table_name(NOTIFICATIONS_TABLE_NAME.clone())
            .key(NOTIFICATION_ID_FIELD_PK, id_av)
            .send()
            .await?;

        Ok(())
    }

}
