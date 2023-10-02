

use serde::{Deserialize,Serialize};
use url::Url;
use uuid::Uuid;

#[derive(Deserialize,Serialize, Debug)]
pub struct MatchAPISimilarItem {
    pub asset_id: Uuid,
    pub frame_id: Uuid,
    pub frame_url: String,
    pub frame_second: f32,
    pub asset_url: Option<Url>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MatchAPIResponse {
    pub similars: Vec<MatchAPISimilarItem>,
    pub next_token: String 
}