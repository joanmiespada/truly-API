use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fmt, str::FromStr};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct License {
    id: Uuid,
    creation_time: DateTime<Utc>,
    last_update_time: DateTime<Utc>,
    asset_id: Uuid,
    version: u8,

    right_to_free_distribute: bool,
    if_you_distribute_mention_me: bool,

    right_to_modify: bool,
    if_you_modify_mention_me: bool,

    right_to_use_broadcast_media: bool,
    right_to_use_press_media: bool,

    rights: Vec<Royalty>,

    status: LicenseStatus,
}

impl License {
    pub fn new() -> License {
        License {
            id: Uuid::new_v4(),
            creation_time: Utc::now(),
            last_update_time: Utc::now(),
            asset_id: Uuid::nil(),
            version: 0,

            right_to_free_distribute: false,
            if_you_distribute_mention_me: false,

            right_to_modify: false,
            if_you_modify_mention_me: false,

            right_to_use_broadcast_media: false,
            right_to_use_press_media: false,

            rights: Vec::new(),

            status: LicenseStatus::Enabled,
        }
    }

    // Getter for id
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    // Setter for id
    pub fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    // Getter for creation_time
    pub fn creation_time(&self) -> &DateTime<Utc> {
        &self.creation_time
    }

    // Setter for creation_time
    pub fn set_creation_time(&mut self, creation_time: DateTime<Utc>) {
        self.creation_time = creation_time;
    }

    // Getter for last_update_time
    pub fn last_update_time(&self) -> &DateTime<Utc> {
        &self.last_update_time
    }

    // Setter for last_update_time
    pub fn set_last_update_time(&mut self, last_update_time: DateTime<Utc>) {
        self.last_update_time = last_update_time;
    }

    // Getter for asset_id
    pub fn asset_id(&self) -> &Uuid {
        &self.asset_id
    }

    // Setter for asset_id
    pub fn set_asset_id(&mut self, asset_id: Uuid) {
        self.asset_id = asset_id;
    }

    // Getter for version
    pub fn version(&self) -> u8 {
        self.version
    }

    // Setter for version
    pub fn set_version(&mut self, version: u8) {
        self.version = version;
    }

    // Getter for right_to_free_distribute
    pub fn right_to_free_distribute(&self) -> bool {
        self.right_to_free_distribute
    }

    // Setter for right_to_free_distribute
    pub fn set_right_to_free_distribute(&mut self, right_to_free_distribute: bool) {
        self.right_to_free_distribute = right_to_free_distribute;
    }

    // Getter for if_you_distribute_mention_me
    pub fn if_you_distribute_mention_me(&self) -> bool {
        self.if_you_distribute_mention_me
    }

    // Setter for if_you_distribute_mention_me
    pub fn set_if_you_distribute_mention_me(&mut self, if_you_distribute_mention_me: bool) {
        self.if_you_distribute_mention_me = if_you_distribute_mention_me;
    }

    // Getter for right_to_modify
    pub fn right_to_modify(&self) -> bool {
        self.right_to_modify
    }

    // Setter for right_to_modify
    pub fn set_right_to_modify(&mut self, right_to_modify: bool) {
        self.right_to_modify = right_to_modify;
    }

    // Getter for if_you_modify_mention_me
    pub fn if_you_modify_mention_me(&self) -> bool {
        self.if_you_modify_mention_me
    }

    // Setter for if_you_modify_mention_me
    pub fn set_if_you_modify_mention_me(&mut self, if_you_modify_mention_me: bool) {
        self.if_you_modify_mention_me = if_you_modify_mention_me;
    }

    // Getter for right_to_use_broadcast_media
    pub fn right_to_use_broadcast_media(&self) -> bool {
        self.right_to_use_broadcast_media
    }

    // Setter for right_to_use_broadcast_media
    pub fn set_right_to_use_broadcast_media(&mut self, right_to_use_broadcast_media: bool) {
        self.right_to_use_broadcast_media = right_to_use_broadcast_media;
    }

    // Getter for right_to_use_press_media
    pub fn right_to_use_press_media(&self) -> bool {
        self.right_to_use_press_media
    }

    // Setter for right_to_use_press_media
    pub fn set_right_to_use_press_media(&mut self, right_to_use_press_media: bool) {
        self.right_to_use_press_media = right_to_use_press_media;
    }

    // Getter for rights
    pub fn rights(&self) -> &Vec<Royalty> {
        &self.rights
    }

    // Setter for rights
    pub fn set_rights(&mut self, rights: Vec<Royalty>) {
        self.rights = rights;
    }

    pub fn status(&self) -> &LicenseStatus {
        &self.status
    }

    // Setter for rights
    pub fn set_status(&mut self, new_status: LicenseStatus) {
        self.status = new_status;
    }
}

impl Default for License {
    fn default() -> License {
        License::new()
    }
}

impl fmt::Display for License {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", json!(self).to_string())
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum LicenseStatus {
    Enabled,
    Disabled,
}

impl fmt::Display for LicenseStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LicenseStatus::Enabled => write!(f, "Enabled"),
            LicenseStatus::Disabled => write!(f, "Disabled"),
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct ParseLicenseStatusError;
impl FromStr for LicenseStatus {
    type Err = ParseLicenseStatusError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "Enabled" => Ok(LicenseStatus::Enabled),
            "Disabled" => Ok(LicenseStatus::Disabled),
            _ => Err(ParseLicenseStatusError),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Royalty {
    pub price: f32,
    pub location: String,
}
