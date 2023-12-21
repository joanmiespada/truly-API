use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct FacebookAPIError(pub String);

impl std::error::Error for FacebookAPIError{}

impl Display for FacebookAPIError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Facebook Api error: {}", self.0)
    }
}