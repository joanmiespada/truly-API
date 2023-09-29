use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateHashes {
    pub url_file: Url,
    pub asset_id: Uuid,
}

impl std::fmt::Display for CreateHashes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "url: {} asset_id: {}",
            self.url_file.to_string(),
            self.asset_id.to_string()
        )
    }
}