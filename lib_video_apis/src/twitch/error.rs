use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct TwitchAPIError(pub String);

impl std::error::Error for TwitchAPIError{}

impl Display for TwitchAPIError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TwitchApi error: {}", self.0)
    }
}