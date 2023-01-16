// use chrono::{DateTime, Utc};
// use serde::{Deserialize, Serialize};
// use serde_json::json;
// use std::fmt;
// use uuid::Uuid;
// use validator::Validate;


// #[derive(Clone, Serialize,Validate, Deserialize, Debug)]
// pub struct Owner {
//     asset_id: Uuid,
//     #[validate(length( max=100))]
//     user_id: String,
//     creation_time: DateTime<Utc>,
// }

// impl fmt::Display for Owner {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//             write!(f,"{}", json!(self).to_string())
//     }
// }

// impl Owner {
//     pub fn new() -> Owner {
//         Owner {
//             asset_id: Uuid::nil(),
//             creation_time: Utc::now(),
//             user_id: String::new()
//         }
//     }

//     pub fn asset_id(&self) -> &Uuid {
//         &self.asset_id
//     }
//     pub fn set_asset_id(&mut self, val: &Uuid) {
//         self.asset_id = val.clone()
//     }
//     pub fn creation_time(&self) -> &DateTime<Utc> {
//         &self.creation_time
//     }
//     pub fn set_creation_time(&mut self, val: &DateTime<Utc>) {
//         self.creation_time = val.clone()
//     }
//     pub fn user_id(&self) -> &String {
//         &self.user_id
//     }
//     pub fn set_user_id(&mut self, val: &String) {
//         self.user_id = val.clone()
//     }
// }