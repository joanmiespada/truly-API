
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;
use std::{fmt, str::FromStr};
use uuid::Uuid;
use validator::Validate;


#[derive(Clone, Serialize, Validate, Deserialize, Debug)]
pub struct Asset {
    id: Uuid,
    creation_time: DateTime<Utc>,
    last_update_time: DateTime<Utc>,
    url: Option<Url>,
    status: AssetStatus,

    latitude: Option<f64>,
    longitude: Option<f64>,
    #[validate(length( max=1000))]
    hash: Option<String>,
    #[validate(length( max=1000))]
    license: Option<String>,

}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f,"{}", json!(self).to_string())
    }
}

impl Asset {
    pub fn new() -> Asset {
        Asset {
            id: Uuid::nil(),
            creation_time: Utc::now(),
            last_update_time: Utc::now(),
            url: None,
            status: AssetStatus::Enabled,
            hash: None,
            latitude: None,
            longitude: None,
            license: None
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }
    pub fn set_id(&mut self, val: &Uuid) {
        self.id = val.clone()
    }
    pub fn creation_time(&self) -> &DateTime<Utc> {
        &self.creation_time
    }
    pub fn set_creation_time(&mut self, val: &DateTime<Utc>) {
        self.creation_time = val.clone()
    }
    pub fn last_update_time(&self) -> &DateTime<Utc> {
        &self.last_update_time
    }
    pub fn set_last_update_time(&mut self, val: &DateTime<Utc>) {
        self.last_update_time = val.clone()
    }
    pub fn url(&self) -> &Option<Url> {
        &self.url
    }
    pub fn set_url(&mut self, val: &Option<Url>) {
        self.url = val.clone()
    }
    pub fn state(&self) -> &AssetStatus {
        &self.status
    }
    pub fn set_state(&mut self, val: &AssetStatus) {
        self.status = val.clone()
    }

    pub fn hash(&self) -> &Option<String> {
        &self.hash
    }
    pub fn set_hash(&mut self, val: &Option<String>) {
        self.hash = val.clone()
    }

    pub fn longitude(&self) -> &Option<f64> {
        &self.longitude
    }
    pub fn set_longitude(&mut self, val: &Option<f64>) {
        self.longitude = val.clone()
    }

    pub fn latitude(&self) -> &Option<f64> {
        &self.latitude
    }
    pub fn set_latitude(&mut self, val: &Option<f64>) {
        self.latitude = val.clone()
    }

    pub fn license(&self) -> &Option<String> {
        &self.license
    }
    pub fn set_license(&mut self, val: &Option<String>) {
        self.license = val.clone()
    }

}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
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

impl fmt::Display for AssetStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AssetStatus::Enabled => write!(f, "Enabled"),
            AssetStatus::Disabled => write!(f, "Disabled"),
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct ParseAssetStatusError;
impl FromStr for AssetStatus {
    type Err = ParseAssetStatusError ;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "Enabled" => Ok(AssetStatus::Enabled),
            "Disabled" => Ok(AssetStatus::Disabled),
            _ => Err(ParseAssetStatusError), 
        }
    }
}