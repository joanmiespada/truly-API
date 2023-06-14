use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;
use validator::Validate;

//structure to communicate from VideoAPI: sns_topic video in
#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct CreateShorter {
    pub url_file: Url,
    pub asset_id: Uuid,
    #[validate(length(max = 100))]
    pub user_id: String,
    #[validate(length(max = 2000))]
    pub hash: String,
    #[validate(length(max = 20))]
    pub hash_algorithm: String,
    pub keep_original: bool,
}
