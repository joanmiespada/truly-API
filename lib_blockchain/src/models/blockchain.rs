use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
use url::Url;
use validator::Validate;

#[derive(Clone, Serialize, Validate, Deserialize, Debug)]
pub struct Blockchain {
    id: String,
    url: Url,
    api_key: String,
    confirmations: u16,
    explorer: Url,
    explorer_api_key: String,
}

impl fmt::Display for Blockchain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", json!(self).to_string())
    }
}

impl Blockchain {
    pub fn new(
        id: String,
        url: Url,
        api_key: String,
        confirmations: u16,
        explorer: Url,
        explorer_api_key: String,
    ) -> Blockchain {
        Blockchain {
            id,
            url,
            api_key,
            confirmations,
            explorer,
            explorer_api_key,
        }
    }

    pub fn id(&self) -> &String {
        &self.id
    }
    pub fn set_id(&mut self, val: &String) {
        self.id = val.clone()
    }

    pub fn url(&self) -> &Url {
        &self.url
    }
    pub fn set_url(&mut self, val: &Url) {
        self.url = val.clone()
    }

    pub fn api_key(&self) -> &String {
        &self.api_key
    }
    pub fn set_api_key(&mut self, val: &String) {
        self.api_key = val.clone()
    }
    pub fn confirmations(&self) -> &u16 {
        &self.confirmations
    }
    pub fn set_confirmations(&mut self, val: &u16) {
        self.confirmations = val.clone()
    }

    pub fn explorer(&self) -> &Url {
        &self.explorer
    }
    pub fn set_explorer(&mut self, val: &Url) {
        self.explorer = val.clone()
    }
    pub fn explorer_api_key(&self) -> &String {
        &self.explorer_api_key
    }
    pub fn set_explorer_api_key(&mut self, val: &String) {
        self.explorer_api_key = val.clone()
    }
}
