

use serde::{Deserialize,Serialize};
use url::Url;
use uuid::Uuid;

#[derive(Deserialize,Serialize, Debug)]
pub struct MatchAPISimilarItem {
    pub asset_id: Uuid,
    pub frame_id: Uuid,
    pub frame_url: String,
    pub frame_second: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MatchAPIResponse {
    pub similars: Vec<MatchAPISimilarItem>,
    pub next_token: String 
}

#[derive(Deserialize,Serialize, Debug, Clone)]
pub struct SimilarItem {
    pub asset_id: Uuid,
    pub frame_id: Uuid,
    pub frame_url: String,
    pub frame_second: String,
    pub asset_url: Option<Url>
}

#[derive(Deserialize, Serialize, Debug,  Clone)]
pub struct SimilarResponse {
    pub similars: Vec<SimilarItem>,
    pub next_token: String 
}