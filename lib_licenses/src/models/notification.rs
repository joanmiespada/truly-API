use derive_builder::Builder;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::fmt;


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Notification {
    id: Uuid,
    creation_time: DateTime<Utc>,
    source_type: Option<String>,
    origin_hash_id: Option<Uuid>,
    origin_hash_type: Option<String>,
    origin_frame_id: Option<Uuid>,
    origin_frame_second: Option<f64>,
    origin_frame_url: Option<String>,
    origin_asset_id: Option<Uuid>,
    similar_frame_id: Option<Uuid>,
    similar_frame_second: Option<f64>,
    similar_frame_url: Option<String>,
    similar_asset_id: Option<Uuid>,
}

impl fmt::Display for Notification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // You can format it the way you want, here's a basic representation
        write!(f, "{:?}", self)
    }
}

impl Notification {
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn set_id(&mut self, id: Uuid){
        self.id = id;
    }

    pub fn creation_time(&self) -> &DateTime<Utc> {
        &self.creation_time
    } 
    pub fn set_creation_time(&mut self, creation_time: DateTime<Utc>) {
        self.creation_time = creation_time;
    } 
    pub fn source_type(&self) -> &Option<String> {
        &self.source_type
    }
    pub fn set_source_type(&mut self, source_type: String) {
        self.source_type = Some(source_type);
    }
    pub fn origin_hash_id(&self) -> &Option<Uuid> {
        &self.origin_hash_id
    }
    pub fn set_origin_hash_id(&mut self, id: Uuid) {
        self.origin_hash_id = Some(id);
    } 
    pub fn origin_hash_type(&self) -> &Option<String> {
        &self.origin_hash_type
    }
    pub fn set_origin_hash_type(&mut self, origin_hash_type: String) {
        self.origin_hash_type = Some(origin_hash_type);
    } 
    pub fn origin_frame_id(&self) -> &Option<Uuid> {
        &self.origin_frame_id
    }
    pub fn set_origin_frame_id(&mut self, id: Uuid) {
        self.origin_frame_id = Some(id);
    }
    pub fn origin_frame_second(&self) -> &Option<f64> {
        &self.origin_frame_second
    }
    pub fn set_origin_frame_second(&mut self, origin_frame_second: f64) {
        self.origin_frame_second = Some(origin_frame_second);
    }
    pub fn origin_asset_id(&self) -> &Option<Uuid> {
        &self.origin_asset_id
    }
    pub fn set_origin_asset_id(&mut self, id: Uuid) {
        self.origin_asset_id = Some(id);
    }
    pub fn similar_frame_id(&self) -> &Option<Uuid> {
        &self.similar_frame_id
    }
    pub fn set_similar_frame_id(&mut self, id: Uuid) {
        self.similar_frame_id = Some(id);
    }
    pub fn similar_frame_second(&self) -> &Option<f64> {
        &self.similar_frame_second
    }
    pub fn set_similar_frame_second(&mut self, similar_frame_second: f64) {
        self.similar_frame_second = Some(similar_frame_second);
    }
    pub fn similar_frame_url(&self) -> &Option<String> {
        &self.similar_frame_url
    }
    pub fn set_similar_frame_url(&mut self, similar_frame_url: String) {
        self.similar_frame_url = Some(similar_frame_url);
    }
    pub fn similar_asset_id(&self) -> &Option<Uuid> {
        &self.similar_asset_id
    }
    pub fn set_similar_asset_id(&mut self, id: Uuid) {
        self.similar_asset_id = Some(id);
    }
    pub fn origin_frame_url(&self) -> &Option<String> {
        &self.origin_frame_url
    }
    pub fn set_origin_frame_url(&mut self, origin_frame_url: String) {
        self.origin_frame_url = Some(origin_frame_url);
    }

}

#[derive(Builder, Debug, Clone,Default)]
#[builder(setter(into))]
pub struct NotificationBuilder {
    id: Uuid,
    #[builder(default="Utc::now()")]
    creation_time: DateTime<Utc>,
    source_type: Option<String>,
    origin_hash_id: Option<Uuid>,
    origin_hash_type: Option<String>,
    origin_frame_id: Option<Uuid>,
    origin_frame_second: Option<f64>,
    origin_frame_url: Option<String>,
    origin_asset_id: Option<Uuid>,
    similar_frame_id: Option<Uuid>,
    similar_frame_second: Option<f64>,
    similar_frame_url: Option<String>,
    similar_asset_id: Option<Uuid>,
}

impl NotificationBuilder {
    pub fn build(&self) -> Notification {
        Notification {
            id: self.id.clone(),
            creation_time: self.creation_time.clone(),
            source_type: self.source_type.clone(),
            origin_hash_id: self.origin_hash_id.clone(),
            origin_hash_type: self.origin_hash_type.clone(),
            origin_frame_id: self.origin_frame_id.clone(),
            origin_frame_second: self.origin_frame_second.clone(),
            origin_frame_url: self.origin_frame_url.clone(),
            origin_asset_id: self.origin_asset_id.clone(),
            similar_frame_id: self.similar_frame_id.clone(),
            similar_frame_second: self.similar_frame_second,
            similar_frame_url: self.similar_frame_url.clone(),
            similar_asset_id: self.similar_asset_id.clone(),
        }
    }
}



