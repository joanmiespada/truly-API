use serde::{Serialize, Deserialize};
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
    #[validate(length(max = 200))]
    pub hash: String,
    pub keep_original: bool
}

//structure to communicate from VideoAPI: sns_topic video out
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoResult {
    pub url_file: Url, //temporally bucket
    pub asset_id: Uuid,
    pub user_id: String,
    pub hash: String,
    pub counter: u64,
    pub shorter: String,
    pub video_op: Option<bool>,
    pub video_error: Option<String>,
    pub video_licensed_asset_id: Option<Uuid>,
    pub video_licensed : Option<Url>, //final and permanent bucket
    pub video_licensed_hash: Option<String>,
    pub keep_original: bool,
    pub video_original : Option<Url>, //final and permanent bucket
    pub video_original_hash: Option<String>, 
}

