use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fmt, str::FromStr};
use url::Url;
use uuid::Uuid;
use validator::Validate;
use web3::types::H256;

use super::video::VideoProcessStatus;

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
    license: Option<String>,

    //#[validate(length( max=1000))]
    last_minted_tx: Option<H256>,
    mint_status: MintingStatus,

    counter: Option<u64>,
    shorter: Option<String>,

    video_licensing_error: Option<String>,
    video_licensing_status: VideoLicensingStatus,

    video_process_status: Option<VideoProcessStatus>,

    father: Option<Uuid>,

    source: Option<SourceType>,
    source_details: Option<String>,
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
            latitude: None,
            longitude: None,
            license: None,
            last_minted_tx: None,
            mint_status: MintingStatus::NeverMinted,
            shorter: None,
            counter: None,
            father: None,
            video_licensing_error: None,
            video_licensing_status: VideoLicensingStatus::NeverStarted,
            video_process_status: None,
            source: None,
            source_details: None,
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

    pub fn license(&self) -> &Option<String> {
        &self.license
    }
    pub fn set_license(&mut self, val: &Option<String>) {
        self.license = val.clone()
    }
    pub fn minted_tx(&self) -> &Option<H256> {
        &self.last_minted_tx
    }
    pub fn set_minted_tx(&mut self, val: &Option<H256>) {
        self.last_minted_tx = val.clone()
    }
    pub fn mint_status(&self) -> &MintingStatus {
        &self.mint_status
    }
    pub fn set_minted_status(&mut self, val: MintingStatus) {
        self.mint_status = val.clone()
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
    /*
        pub fn (&self) -> &Option<> {
            &self.
        }
        pub fn set_(&mut self, val: &Option<>) {
            self. = val.clone()
        }
    */
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
pub enum MintingStatus {
    NeverMinted,
    Scheduled,
    Started,
    CompletedSuccessfully,
    Error,
}

impl fmt::Display for MintingStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MintingStatus::Scheduled => write!(f, "Scheduled"),
            MintingStatus::Started => write!(f, "Started"),
            MintingStatus::CompletedSuccessfully => write!(f, "Completed successfully"),
            MintingStatus::Error => write!(f, "Error"),
            MintingStatus::NeverMinted => write!(f, "Never minted"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MintinStatusParseError;

impl FromStr for MintingStatus {
    type Err = MintinStatusParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Scheduled" => Ok(MintingStatus::Scheduled),
            "Started" => Ok(MintingStatus::Started),
            "Completed successfully" => Ok(MintingStatus::CompletedSuccessfully),
            "Error" => Ok(MintingStatus::Error),
            "Never minted" => Ok(MintingStatus::NeverMinted),
            _ => Err(MintinStatusParseError),
        }
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
