use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct YoutubeAPIError(pub String);

impl std::error::Error for YoutubeAPIError{}

impl Display for YoutubeAPIError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "YoutubeApi error: {}", self.0)
    }
}