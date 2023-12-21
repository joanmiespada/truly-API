use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct InstagramAPIError(pub String);

impl std::error::Error for InstagramAPIError{}

impl Display for InstagramAPIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Instagram API error: {}", self.0)
    }
}