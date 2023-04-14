use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use validator::Validate;

#[derive(Clone, Serialize, Validate, Deserialize, Debug)]
pub struct KeyPair {
    #[validate(length(max = 100))]
    user_id: String,
    creation_time: DateTime<Utc>,
    last_update_time: DateTime<Utc>,

    address: String,
    public_key: String,
    private_key: String,
}

impl fmt::Display for KeyPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Key user id: {} address: {}", self.user_id, self.address)
    }
}

impl KeyPair {
    pub fn new() -> KeyPair {
        KeyPair {
            user_id: "".to_string(),
            creation_time: Utc::now(),
            last_update_time: Utc::now(),
            address: "".to_string(),
            public_key: "".to_string(),
            private_key: "".to_string(),
        }
    }

    pub fn creation_time(&self) -> &DateTime<Utc> {
        &self.creation_time
    }
    pub fn set_creation_time(&mut self, val: &DateTime<Utc>) {
        self.creation_time = val.clone()
    }
    pub fn user_id(&self) -> &String {
        &self.user_id
    }
    pub fn set_user_id(&mut self, val: &String) {
        self.user_id = val.clone()
    }
    pub fn last_update_time(&self) -> &DateTime<Utc> {
        &self.last_update_time
    }
    pub fn set_last_update_time(&mut self, val: &DateTime<Utc>) {
        self.last_update_time = val.clone()
    }
    pub fn address(&self) -> &String {
        &self.address
    }
    pub fn set_address(&mut self, val: &String) {
        self.address = val.clone()
    }
    pub fn public_key(&self) -> &String {
        &self.public_key
    }
    pub fn set_public_key(&mut self, val: &String) {
        self.public_key = val.clone()
    }
    pub fn private_key(&self) -> &String {
        &self.private_key
    }
    pub fn set_private_key(&mut self, val: &String) {
        self.private_key = val.clone()
    }
}
