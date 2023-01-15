
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
use uuid::Uuid;
use validator::Validate;
use http::Uri;


#[derive(Clone, Serialize, Validate, Deserialize, Debug)]
pub struct Asset {
    asset_id: Uuid,
    creation_time: DateTime<Utc>,
    #[serde(with = "http_serde::uri")]
    url: Uri,
    status: AssetStatus,
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f,"{}", json!(self).to_string())
    }
}

impl Asset {
    pub fn new() -> Asset {
        Asset {
            asset_id: Uuid::nil(),
            creation_time: Utc::now(),
            url: Uri::from_static(""),
            status: AssetStatus::Enabled
        }
    }

    pub fn asset_id(&self) -> &Uuid {
        &self.asset_id
    }
    pub fn set_asset_id(&mut self, val: &Uuid) {
        self.asset_id = val.clone()
    }
    pub fn creation_time(&self) -> &DateTime<Utc> {
        &self.creation_time
    }
    pub fn set_creation_time(&mut self, val: &DateTime<Utc>) {
        self.creation_time = val.clone()
    }
    pub fn url(&self) -> &Uri {
        &self.url
    }
    pub fn set_url(&mut self, val: &Uri) {
        self.url = val.clone()
    }
    pub fn state(&self) -> &AssetStatus {
        &self.status
    }
    pub fn set_state(&mut self, val: &AssetStatus) {
        self.status = val.clone()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum AssetStatus {
    Enabled,
    Disabled,
}

impl AssetStatus{
    pub fn is_disabled(&self) -> bool {
        match *self {
            AssetStatus::Disabled => true,
            _ => false,
        }
    }
}