
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Ledge{
    pub tx: String,
    pub digest: String,
    creation_time: DateTime<Utc>
}

impl Default for Ledge{
    fn default() -> Self {
        Self { tx: Default::default(), digest: Default::default(), creation_time: Default::default() }
    }
}