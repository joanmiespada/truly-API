// //DON'T EDIT!!! 
// //file copied from VideoApi repo. lib_video/src/video.rs

// use std::{fmt, str::FromStr};

// use serde::{Deserialize, Serialize};
// use url::Url;
// use uuid::Uuid;

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct VideoResult {
//     pub url_file: Url, //temporally bucket
//     pub asset_id: Uuid,
//     pub user_id: String,
//     pub hash: String,
//     pub hash_algorithm: String,
//     pub counter: u64,
//     pub shorter: String,
//     pub video_op: Option<bool>,
//     pub video_error: Option<String>,
//     pub video_licensed_asset_id: Option<Uuid>,
//     pub video_licensed: Option<Url>, //final and permanent bucket
//     pub video_licensed_hash: Option<String>,
//     pub video_licensed_hash_algorithm: Option<String>,
//     pub keep_original: bool,
//     pub video_original: Option<Url>, //final and permanent bucket
//     pub video_original_hash: Option<String>,
//     pub video_original_hash_algorithm: Option<String>,
//     pub video_process_status: Option<VideoProcessStatus>,
// }

// #[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
// pub enum VideoProcessStatus {
//     Started,
//     Downloaded,
//     HashVerified,
//     ApplyingLicense,
//     Uploading,
//     UploadedLicensed,
//     UploadedOriginal,
//     CompletedSuccessfully,
//     Error,
// }

// impl fmt::Display for VideoProcessStatus {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             VideoProcessStatus::Started => write!(f, "Started"),
//             VideoProcessStatus::CompletedSuccessfully => write!(f, "Completed successfully"),
//             VideoProcessStatus::Error => write!(f, "Error"),
//             VideoProcessStatus::HashVerified => write!(f, "Hash verified"),
//             VideoProcessStatus::ApplyingLicense => write!(f, "Applying license"),
//             VideoProcessStatus::Uploading => write!(f, "Uploading"),
//             VideoProcessStatus::UploadedLicensed => write!(f, "Uploaded license asset"),
//             VideoProcessStatus::UploadedOriginal => write!(f, "Uploaded original asset"),
//             VideoProcessStatus::Downloaded => write!(f, "Downloaded"),
//         }
//     }
// }

// #[derive(Debug, PartialEq, Eq)]
// pub struct VideoProcessStatusParseError;

// impl FromStr for VideoProcessStatus {
//     type Err = VideoProcessStatusParseError;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             "Started" => Ok(VideoProcessStatus::Started),
//             "Completed successfully" => Ok(VideoProcessStatus::CompletedSuccessfully),
//             "Error" => Ok(VideoProcessStatus::Error),
//             "Hash verified" => Ok(VideoProcessStatus::HashVerified),
//             "Applying license" => Ok(VideoProcessStatus::ApplyingLicense),
//             "Uploading" => Ok(VideoProcessStatus::Uploading),
//             "Uploaded license asset" => Ok(VideoProcessStatus::UploadedLicensed),
//             "Uploaded original asset" => Ok(VideoProcessStatus::UploadedOriginal),
//             "Downloaded" => Ok(VideoProcessStatus::Downloaded),
//             _ => Err(VideoProcessStatusParseError),
//         }
//     }
// }

// impl fmt::Display for VideoProcessStatusParseError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "error parsing video process status")
//     }
// }
