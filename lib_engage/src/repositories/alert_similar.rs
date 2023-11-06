use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, types::{AttributeValue,Put,TransactWriteItem}};
use lib_config::{config::Config, timing::{from_iso8601, iso8601}};
use uuid::Uuid;
use crate::{models::alert_similar::{AlertSimilar, AlertSimilarBuilder}, errors::alert_similar::AlertSimilarError};
use std::{collections::HashMap, str::FromStr, time::{SystemTime, Duration}};
use lib_config::result::ResultE;
use chrono::{Utc, Local};
use lib_config::pagination::{pagination_encode_token, pagination_decode_token };
use lib_config::pagination::AttributeValueWrapper;

use super::schema_alert_similar::{ALERT_SIMILARS_TABLE_NAME, ALERT_SIMILAR_ID_FIELD_PK, CREATION_TIME, TIME_INDEX_NAME};

pub const LAST_UPDATE_TIME: &str = "last_update_time";
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
pub trait AlertSimilarRepository{
    async fn add(&self, alert: &AlertSimilar) -> ResultE<()>;
    async fn get(&self, alert_id: Uuid ) -> ResultE<Option<AlertSimilar>>;
    async fn update(&self, alert: &AlertSimilar) -> ResultE<()>;
    async fn delete(&self, alert_id: Uuid) -> ResultE<()>;
    async fn check_if_exists(&self, id:Uuid) -> ResultE<bool>;
    async fn get_all_by_time(&self, starting_at: SystemTime, window:Duration, token: Option<String>, limit: Option<u32> ) -> ResultE<(Vec<AlertSimilar>, Option<String> )>;
    async fn get_all(&self, token: Option<String>, limit: Option<u32> ) -> ResultE<(Vec<AlertSimilar>, Option<String> )>;

}


pub struct AlertSimilarRepo {
    client: Client,
    default_page_size: Option<u32>,
    pagination_token: Option<String>
}

impl AlertSimilarRepo {

    pub fn new(conf: &Config) -> Self {
        AlertSimilarRepo{
            client: Client::new(conf.aws_config()),
            default_page_size: conf.env_vars().default_page_size(),
            pagination_token: conf.env_vars().pagination_token_encoder()
        }
    }

    async fn add_or_update(&self, alert: &AlertSimilar) -> ResultE<()> {
        
        let id_av = AttributeValue::S(alert.id().to_string());
        let creation_time_av = AttributeValue::S(iso8601(alert.creation_time()));
        let last_update_time_av = AttributeValue::S(iso8601( &Utc::now() ));
        
        let mut items = Put::builder();
        
        items = items.item(ALERT_SIMILAR_ID_FIELD_PK, id_av);
        items = items.item(CREATION_TIME,creation_time_av);
        items = items.item( LAST_UPDATE_TIME,last_update_time_av);

        if let Some(source_type) = alert.source_type() {
            let source_type_av = AttributeValue::S(source_type.to_string());
            items = items.item(SOURCE_TYPE, source_type_av);
        }

        if let Some(origin_hash_id) = alert.origin_hash_id() {
            let origin_hash_id_av = AttributeValue::S(origin_hash_id.to_string());
            items = items.item(ORIGIN_HASH_ID, origin_hash_id_av);
        }

        if let Some(origin_hash_type) = alert.origin_hash_type() {
            let origin_hash_type_av = AttributeValue::S(origin_hash_type.to_string());
            items = items.item(ORIGIN_HASH_TYPE, origin_hash_type_av);
        }
        
        if let Some(origin_frame_id) = alert.origin_frame_id() {
            let origin_frame_id_av = AttributeValue::S(origin_frame_id.to_string());
            items = items.item(ORIGIN_FRAME_ID, origin_frame_id_av);
        }

        if let Some(origin_frame_second) = alert.origin_frame_second() {
            let origin_frame_second_av = AttributeValue::N(origin_frame_second.to_string());
            items = items.item(ORIGIN_FRAME_SECOND, origin_frame_second_av);
        }

        if let Some(origin_frame_url) = alert.origin_frame_url() {
            let origin_frame_url_av = AttributeValue::S(origin_frame_url.to_string());
            items = items.item(ORIGIN_FRAME_URL, origin_frame_url_av);
        }

        if let Some(origin_asset_id) = alert.origin_asset_id() {
            let origin_asset_id_av = AttributeValue::S(origin_asset_id.to_string());
            items = items.item(ORIGIN_ASSET_ID, origin_asset_id_av);
        }

        if let Some(similar_frame_id) = alert.similar_frame_id() {
            let similar_frame_id_av = AttributeValue::S(similar_frame_id.to_string());
            items = items.item(SIMILAR_FRAME_ID, similar_frame_id_av);
        }

        if let Some(similar_frame_second) = alert.similar_frame_second() {
            let similar_frame_second_av = AttributeValue::N(similar_frame_second.to_string());
            items = items.item(SIMILAR_FRAME_SECOND, similar_frame_second_av);
        }

        if let Some(similar_asset_id) = alert.similar_asset_id() {
            let similar_asset_id_av = AttributeValue::S(similar_asset_id.to_string());
            items = items.item(SIMILAR_ASSET_ID, similar_asset_id_av);
        }

        if let Some(similar_frame_url) = alert.similar_frame_url() {
            let similar_frame_url_av = AttributeValue::S(similar_frame_url.to_string());
            items = items.item(SIMILAR_FRAME_URL, similar_frame_url_av);
        }

        let request = self.client.transact_write_items().transact_items(
            TransactWriteItem::builder()
                .put(items.table_name(ALERT_SIMILARS_TABLE_NAME.clone()).build().unwrap())
                .build(),
        );
        
        match request.send().await {
            Ok(_) => Ok(()),
            Err(e) => {
                    let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                log::error!("{}",mssag);
                return Err( AlertSimilarError::AlertSimilarDynamoDBError(e.into()).into());
                
            },
        }
    }

    


}
#[async_trait]
impl AlertSimilarRepository for AlertSimilarRepo {
    async fn add(&self, alert: &AlertSimilar) -> ResultE<()> {

        if self.check_if_exists(alert.id().clone()).await? {
            return Err(AlertSimilarError::AlertSimilarAlreadyExists(alert.id().clone()).into( ));
        }

        self.add_or_update(alert).await
    }

    async fn get(&self, alert_id: Uuid) -> ResultE<Option<AlertSimilar>> {

        let id_av = AttributeValue::S(alert_id.to_string());
        let output = self.client.get_item()
            .table_name(ALERT_SIMILARS_TABLE_NAME.clone())
            .key(ALERT_SIMILAR_ID_FIELD_PK, id_av)
            .send()
            .await?;

        if let Some(item) = output.item {
            //let mut alert = AlertSimilarBuilder::default().build()?;
            let alert = mapping_from_doc(&item).unwrap();
            return Ok(Some(alert));
        }
        Ok(None)
    }

    async fn update(&self, alert: &AlertSimilar) -> ResultE<()> {

        if !self.check_if_exists(alert.id().clone()).await? {
            return Err(AlertSimilarError::AlertSimilarNotFound(alert.id().clone()).into( ));
        }
        self.add_or_update(alert).await
    }

    async fn delete(&self, alert_id:Uuid) -> ResultE<()> {

        let id_av = AttributeValue::S(alert_id.to_string());
        self.client.delete_item()
            .table_name(ALERT_SIMILARS_TABLE_NAME.clone())
            .key(ALERT_SIMILAR_ID_FIELD_PK, id_av)
            .send()
            .await?;

        Ok(())
    }

    async fn check_if_exists(&self, id:Uuid) -> ResultE<bool> {
        let id_av = AttributeValue::S(id.to_string());
        let output = self.client.get_item()
            .table_name(ALERT_SIMILARS_TABLE_NAME.clone())
            .key(ALERT_SIMILAR_ID_FIELD_PK, id_av)
            .send()
            .await?;

        if let Some(_item) = output.item {
            return Ok(true);
        }
        Ok(false)
    }
    async fn get_all_by_time(&self, starting_at: SystemTime, window:Duration, token: Option<String>, page_size: Option<u32> ) -> ResultE<(Vec<AlertSimilar>, Option<String> )> {

        let end_time = starting_at + window;

        // Convert SystemTime to a suitable format for DynamoDB, e.g., Unix timestamp
        let start_timestamp = starting_at.duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
        let end_timestamp = end_time.duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
        let start_timestamp_av = AttributeValue::S(start_timestamp.to_string());
        let end_timestamp_av = AttributeValue::S(end_timestamp.to_string());
        
        let limit = page_size.unwrap_or(self.default_page_size.unwrap());

        let mut query_input = aws_sdk_dynamodb::operation::query::QueryInput::builder()
            .table_name(ALERT_SIMILARS_TABLE_NAME.as_str())
            .index_name(TIME_INDEX_NAME)
            .key_condition_expression("#creation_time BETWEEN :start_time AND :end_time")
            .expression_attribute_names("#creation_time".to_string(), CREATION_TIME.to_string() )
            .expression_attribute_values(":start_time".to_string(), start_timestamp_av)
            .expression_attribute_values(":end_time".to_string(), end_timestamp_av)
            .limit(limit.try_into().unwrap() )
            .select(aws_sdk_dynamodb::types::Select::AllProjectedAttributes);

        if let Some(t) = token {

            let maybe_decoded_map = pagination_decode_token::<AttributeValueWrapper>(self.pagination_token.clone(), Some(t))?;
            if let Some(decoded_map) = maybe_decoded_map {
                let converted_map: HashMap<_, AttributeValue> = decoded_map
                    .into_iter()
                    .map(|(key, wrapper)| (key, wrapper.get() ))
                    .collect();
                query_input = query_input.set_exclusive_start_key(Some(converted_map));
            }

            //deserialize token!
            //query_input.set_exclusive_start_key( 
            //    pagination_decode_token::<AttributeValueWrapper>( &self.env_vars, &t)?); 
            // You'd need a function to deserialize the token into a key
        }

        let response = query_input.send_with(&self.client).await?;
        

        let alerts: Vec<AlertSimilar> = response.items() //.unwrap_or(&Vec::new())
        .iter()
        .map(|item|  mapping_from_doc(item).unwrap() ) // Assuming a 'from' implementation exists
        .collect();

        let next_token = if response.last_evaluated_key().is_some() {
            let aux = response.last_evaluated_key();

            match aux{
                None=> None,
                Some(value)=>{
                    let converted_map: HashMap<String, AttributeValueWrapper> = value
                    .into_iter()
                    .map(|(key, att)| (key.clone() , AttributeValueWrapper::new(att ) ))
                    .collect();
            
                    pagination_encode_token::<AttributeValueWrapper>( self.pagination_token.clone(), Some(converted_map) ) // You'd need a function to serialize the key into a token

                }
            }

        } else {
            None
        };

        Ok((alerts, next_token))

    }

    async fn get_all(&self, token: Option<String>, page_size: Option<u32> ) -> ResultE<(Vec<AlertSimilar>, Option<String> )>{

        let limit = page_size.unwrap_or(self.default_page_size.unwrap());

        let mut scan_input = aws_sdk_dynamodb::operation::scan::ScanInput::builder()
            .table_name(ALERT_SIMILARS_TABLE_NAME.as_str())
            .limit(limit.try_into().unwrap() )
            .select(aws_sdk_dynamodb::types::Select::AllAttributes);

        if let Some(t) = token {

            let maybe_decoded_map = pagination_decode_token::<AttributeValueWrapper>(self.pagination_token.clone(), Some(t))?;
            if let Some(decoded_map) = maybe_decoded_map {
                let converted_map: HashMap<_, AttributeValue> = decoded_map
                    .into_iter()
                    .map(|(key, wrapper)| (key, wrapper.get() ))
                    .collect();
                scan_input = scan_input.set_exclusive_start_key(Some(converted_map));
            }
        }

        let response = scan_input.send_with(&self.client).await?;
        

        let alerts: Vec<AlertSimilar> = response.items() //.unwrap_or(&Vec::new())
        .iter()
        .map(|item|  mapping_from_doc(item).unwrap() ) // Assuming a 'from' implementation exists
        .collect();

        let next_token = if response.last_evaluated_key().is_some() {
            let aux = response.last_evaluated_key();

            match aux{
                None=> None,
                Some(value)=>{
                    let converted_map: HashMap<String, AttributeValueWrapper> = value
                    .into_iter()
                    .map(|(key, att)| (key.clone() , AttributeValueWrapper::new(att ) ))
                    .collect();
            
                    pagination_encode_token::<AttributeValueWrapper>( self.pagination_token.clone(), Some(converted_map) ) 

                }
            }

        } else {
            None
        };

        Ok((alerts, next_token))


    }
}

fn mapping_from_doc(doc: &HashMap<String, AttributeValue>) -> ResultE<AlertSimilar> {

    let mut alert = AlertSimilarBuilder::default().build()?;

    let _id = doc.get(ALERT_SIMILAR_ID_FIELD_PK).unwrap();
    let notif_id = _id.as_s().unwrap();
    let notif_uuid = Uuid::from_str(notif_id).unwrap();
    alert.set_id(notif_uuid);

    let creation_time = doc.get(CREATION_TIME).unwrap();
    alert.set_creation_time(from_iso8601(creation_time.as_s().unwrap()));

    let last_update_time = doc.get(LAST_UPDATE_TIME).unwrap();
    alert.set_last_update_time(from_iso8601(last_update_time.as_s().unwrap()) );
        
    if let Some(value) = doc.get(SOURCE_TYPE){
        let val = value.as_s().unwrap();
        alert.set_source_type(val.to_string());
    }

    if let Some(value) = doc.get(ORIGIN_HASH_ID){
        let val = value.as_s().unwrap();
        let uuid = Uuid::from_str(val).unwrap();
        alert.set_origin_hash_id(uuid);
    }

    if let Some(value) = doc.get(ORIGIN_HASH_TYPE){
        let val = value.as_s().unwrap();
        alert.set_origin_hash_type(val.to_string());
    }

    if let Some(value) = doc.get(ORIGIN_FRAME_ID){
        let val = value.as_s().unwrap();
        let uuid = Uuid::from_str(val).unwrap();
        alert.set_origin_frame_id(uuid);
    }

    if let Some(value) = doc.get(ORIGIN_FRAME_SECOND){
        let val = value.as_n().unwrap();
        let second = val.parse::<f64>().unwrap();
        alert.set_origin_frame_second(second);
    }

    if let Some(value) = doc.get(ORIGIN_FRAME_URL){
        let val = value.as_s().unwrap();
        alert.set_origin_frame_url(val.to_string());
    }

    if let Some(value) = doc.get(ORIGIN_ASSET_ID){
        let val = value.as_s().unwrap();
        let uuid = Uuid::from_str(val).unwrap();
        alert.set_origin_asset_id(uuid);
    }

    if let Some(value) = doc.get(SIMILAR_FRAME_ID){
        let val = value.as_s().unwrap();
        let uuid = Uuid::from_str(val).unwrap();
        alert.set_similar_frame_id(uuid);
    }

    if let Some(value) = doc.get(SIMILAR_FRAME_SECOND){
        let val = value.as_n().unwrap();
        let second = val.parse::<f64>().unwrap();
        alert.set_similar_frame_second(second);
    }

    if let Some(value) = doc.get(SIMILAR_ASSET_ID){
        let val = value.as_s().unwrap();
        let uuid = Uuid::from_str(val).unwrap();
        alert.set_similar_asset_id(uuid);
    }

    if let Some(value) = doc.get(SIMILAR_FRAME_URL){
        let val = value.as_s().unwrap();
        alert.set_similar_frame_url(val.to_string());
    }

    Ok(alert)

}