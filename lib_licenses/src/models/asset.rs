use chrono::{DateTime, Utc};
use lib_video_objs::video::VideoProcessStatus;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fmt, str::FromStr};
use url::Url;
use uuid::Uuid;
use validator::Validate;

#[derive(Clone, Serialize, Validate, Deserialize, Debug)]
pub struct Asset {
    id: Uuid,
    creation_time: DateTime<Utc>,
    last_update_time: DateTime<Utc>,
    url: Option<Url>,
    status: AssetStatus,

    latitude: Option<f64>,
    longitude: Option<f64>,
    #[validate(length(max = 1000))]
    hash: Option<String>,
    #[validate(length(max = 1000))]
    hash_algorithm: Option<String>,

    counter: Option<u64>,
    shorter: Option<String>,

    video_licensing_error: Option<String>,
    video_licensing_status: VideoLicensingStatus,

    video_process_status: Option<VideoProcessStatus>,

    father: Option<Uuid>,

    source: Option<SourceType>,
    source_details: Option<String>,

    hash_process_status: Option<HashProcessStatus>,
    hash_process_error_message: Option<String>,
    hash_process_error_stage: Option<String>,
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", json!(self).to_string())
    }
}

impl Asset {
    pub fn new() -> Asset {
        Asset {
            id: Uuid::nil(),
            creation_time: Utc::now(),
            last_update_time: Utc::now(),
            url: None,
            status: AssetStatus::Enabled,
            hash: None,
            hash_algorithm: None,
            latitude: None,
            longitude: None,
            shorter: None,
            counter: None,
            father: None,
            video_licensing_error: None,
            video_licensing_status: VideoLicensingStatus::NeverStarted,
            video_process_status: None,
            source: None,
            source_details: None,
            hash_process_status: None,
            hash_process_error_message: None,
            hash_process_error_stage: None,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }
    pub fn set_id(&mut self, val: &Uuid) {
        self.id = val.clone()
    }
    pub fn creation_time(&self) -> &DateTime<Utc> {
        &self.creation_time
    }
    pub fn set_creation_time(&mut self, val: &DateTime<Utc>) {
        self.creation_time = val.clone()
    }
    pub fn last_update_time(&self) -> &DateTime<Utc> {
        &self.last_update_time
    }
    pub fn set_last_update_time(&mut self, val: &DateTime<Utc>) {
        self.last_update_time = val.clone()
    }
    pub fn url(&self) -> &Option<Url> {
        &self.url
    }
    pub fn set_url(&mut self, val: &Option<Url>) {
        self.url = val.clone()
    }
    pub fn state(&self) -> &AssetStatus {
        &self.status
    }
    pub fn set_state(&mut self, val: &AssetStatus) {
        self.status = val.clone()
    }

    pub fn hash(&self) -> &Option<String> {
        &self.hash
    }
    pub fn set_hash(&mut self, val: &Option<String>) {
        self.hash = val.clone()
    }
    pub fn hash_algorithm(&self) -> &Option<String> {
        &self.hash_algorithm
    }
    pub fn set_hash_algorithm(&mut self, val: &Option<String>) {
        self.hash_algorithm = val.clone()
    }

    pub fn longitude(&self) -> &Option<f64> {
        &self.longitude
    }
    pub fn set_longitude(&mut self, val: &Option<f64>) {
        self.longitude = val.clone()
    }

    pub fn latitude(&self) -> &Option<f64> {
        &self.latitude
    }
    pub fn set_latitude(&mut self, val: &Option<f64>) {
        self.latitude = val.clone()
    }

    pub fn shorter(&self) -> &Option<String> {
        &self.shorter
    }
    pub fn set_shorter(&mut self, val: &Option<String>) {
        self.shorter = val.clone()
    }

    pub fn counter(&self) -> &Option<u64> {
        &self.counter
    }
    pub fn set_counter(&mut self, val: &Option<u64>) {
        self.counter = val.clone()
    }
    pub fn father(&self) -> &Option<Uuid> {
        &self.father
    }
    pub fn set_father(&mut self, val: &Option<Uuid>) {
        self.father = val.clone()
    }
    pub fn video_licensing_error(&self) -> &Option<String> {
        &self.video_licensing_error
    }
    pub fn set_video_licensing_error(&mut self, val: &Option<String>) {
        self.video_licensing_error = val.clone()
    }
    pub fn video_licensing_status(&self) -> &VideoLicensingStatus {
        &self.video_licensing_status
    }
    pub fn set_video_licensing_status(&mut self, val: VideoLicensingStatus) {
        self.video_licensing_status = val.clone()
    }
    pub fn video_process_status(&self) -> &Option<VideoProcessStatus> {
        &self.video_process_status
    }
    pub fn set_video_process_status(&mut self, val: &Option<VideoProcessStatus>) {
        self.video_process_status = val.clone()
    }
    pub fn source(&self) -> &Option<SourceType> {
        &self.source
    }
    pub fn set_source(&mut self, val: &Option<SourceType>) {
        self.source = val.clone()
    }
    pub fn source_details(&self) -> &Option<String> {
        &self.source_details
    }
    pub fn set_source_details(&mut self, val: &Option<String>) {
        self.source_details = val.clone()
    }
    pub fn hash_process_status(&self) -> &Option<HashProcessStatus> {
        &self.hash_process_status
    }
    pub fn set_hash_process_status(&mut self, val: &Option<HashProcessStatus>) {
        self.hash_process_status = val.clone()
    }
    pub fn hash_process_error_stage(&self) -> &Option<String> {
        &self.hash_process_error_stage
    }
    pub fn set_hash_process_error_stage(&mut self, val: &Option<String>) {
        self.hash_process_error_stage = val.clone()
    }

    pub fn hash_process_error_message(&self) -> &Option<String> {
        &self.hash_process_error_message
    }
    pub fn set_hash_process_error_message(&mut self, val: &Option<String>) {
        self.hash_process_error_message = val.clone()
    }
    /*
        pub fn (&self) -> &Option<> {
            &self.
        }
        pub fn set_(&mut self, val: &Option<>) {
            self. = val.clone()
        }
    */
}

impl Default for Asset {
    fn default() -> Asset {
        Asset::new()
    }
}

pub struct AssetBuilder {
    id: Uuid,
    url: Option<Url>,
    hash: Option<String>,
    hash_algorithm: Option<String>,
}

impl AssetBuilder {
    pub fn new() -> AssetBuilder {
        AssetBuilder {
            id: Uuid::nil(),
            url: None,
            hash: None,
            hash_algorithm: None,
        }
    }
    pub fn id(&mut self, id: Uuid) -> &mut AssetBuilder {
        self.id = id.clone();
        self
    }
    pub fn url(&mut self, url: Url) -> &mut AssetBuilder {
        self.url = Some(url.clone());
        self
    }
    pub fn hash(&mut self, hash: &str) -> &mut AssetBuilder {
        self.hash = Some(hash.to_string());
        self
    }
    pub fn hash_algorithm(&mut self, hash_algorithm: &str) -> &mut AssetBuilder {
        self.hash_algorithm = Some(hash_algorithm.to_string());
        self
    }

    pub fn build(&self) -> Asset {
        Asset {
            id: self.id,
            creation_time: Utc::now(),
            last_update_time: Utc::now(),
            url: self.url.clone(),
            status: AssetStatus::Enabled,
            hash: self.hash.clone(),
            hash_algorithm: self.hash_algorithm.clone(),
            latitude: None,
            longitude: None,
            shorter: None,
            counter: None,
            father: None,
            video_licensing_error: None,
            video_licensing_status: VideoLicensingStatus::NeverStarted,
            video_process_status: None,
            source: None,
            source_details: None,
            hash_process_status: None,
            hash_process_error_message: None,
            hash_process_error_stage: None,
        }
    }
}
impl Default for AssetBuilder {
    fn default() -> AssetBuilder {
        AssetBuilder::new()
    }
}

#[derive(Clone, Serialize, Validate, Deserialize, Debug)]
pub struct AssetEnhanced {
    pub asset: Asset,
    pub sons: Vec<Asset>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum AssetStatus {
    Enabled,
    Disabled,
}

impl AssetStatus {
    pub fn is_disabled(&self) -> bool {
        match *self {
            AssetStatus::Disabled => true,
            _ => false,
        }
    }
}

impl fmt::Display for AssetStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AssetStatus::Enabled => write!(f, "Enabled"),
            AssetStatus::Disabled => write!(f, "Disabled"),
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct ParseAssetStatusError;
impl FromStr for AssetStatus {
    type Err = ParseAssetStatusError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "Enabled" => Ok(AssetStatus::Enabled),
            "Disabled" => Ok(AssetStatus::Disabled),
            _ => Err(ParseAssetStatusError),
        }
    }
}

impl fmt::Display for ParseAssetStatusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error parsing asset status type")
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum VideoLicensingStatus {
    NeverStarted,
    Scheduled,
    Started,
    CompletedSuccessfully,
    AlreadyLicensed,
    Error,
}

impl fmt::Display for VideoLicensingStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VideoLicensingStatus::Scheduled => write!(f, "Scheduled"),
            VideoLicensingStatus::Started => write!(f, "Started"),
            VideoLicensingStatus::CompletedSuccessfully => write!(f, "Completed successfully"),
            VideoLicensingStatus::Error => write!(f, "Error"),
            VideoLicensingStatus::NeverStarted => write!(f, "Never started"),
            VideoLicensingStatus::AlreadyLicensed => write!(f, "Already licensed"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct VideoLicensingStatusParseError;

impl FromStr for VideoLicensingStatus {
    type Err = VideoLicensingStatusParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Scheduled" => Ok(VideoLicensingStatus::Scheduled),
            "Started" => Ok(VideoLicensingStatus::Started),
            "Completed successfully" => Ok(VideoLicensingStatus::CompletedSuccessfully),
            "Error" => Ok(VideoLicensingStatus::Error),
            "Never started" => Ok(VideoLicensingStatus::NeverStarted),
            "Already licensed" => Ok(VideoLicensingStatus::AlreadyLicensed),
            _ => Err(VideoLicensingStatusParseError),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum HashProcessStatus {
    NeverStarted,
    Scheduled,
    Started,
    CompletedSuccessfully,
    Error,
}

impl fmt::Display for HashProcessStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HashProcessStatus::Scheduled => write!(f, "Scheduled"),
            HashProcessStatus::Started => write!(f, "Started"),
            HashProcessStatus::CompletedSuccessfully => write!(f, "Completed successfully"),
            HashProcessStatus::Error => write!(f, "Error"),
            HashProcessStatus::NeverStarted => write!(f, "Never started"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct HashProcessStatusParseError;

impl FromStr for HashProcessStatus {
    type Err = HashProcessStatusParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Scheduled" => Ok(HashProcessStatus::Scheduled),
            "Started" => Ok(HashProcessStatus::Started),
            "Completed successfully" => Ok(HashProcessStatus::CompletedSuccessfully),
            "Error" => Ok(HashProcessStatus::Error),
            "Never started" => Ok(HashProcessStatus::NeverStarted),
            _ => Err(HashProcessStatusParseError),
        }
    }
}

impl fmt::Display for HashProcessStatusParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error parsing HashProcessStatus  type")
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum SourceType {
    TrulyApp,
    TrulyWeb,
    TrulyApi,
    Others,
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SourceType::TrulyApp => write!(f, "TrulyApp"),
            SourceType::TrulyWeb => write!(f, "TrulyWeb"),
            SourceType::TrulyApi => write!(f, "TrulyApi"),
            SourceType::Others => write!(f, "Others"),
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct ParseSourceTypeError;
impl FromStr for SourceType {
    type Err = ParseSourceTypeError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "TrulyApp" => Ok(SourceType::TrulyApp),
            "TrulyWeb" => Ok(SourceType::TrulyWeb),
            "TrulyApi" => Ok(SourceType::TrulyApi),
            "Others" => Ok(SourceType::Others),
            _ => Err(ParseSourceTypeError),
        }
    }
}
impl fmt::Display for ParseSourceTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error parsing source type")
    }
}
