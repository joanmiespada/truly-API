use std::fmt::Display;

#[derive(Debug)]
pub struct VideoError(pub String);

impl std::error::Error for VideoError {}

impl Display for VideoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "video error: {}", self.0)
    }
}

#[derive(Debug)]
pub struct VideoNotYetLicensed;

impl std::error::Error for VideoNotYetLicensed {}

impl Display for VideoNotYetLicensed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "video not yet licensed error")
    }
}
