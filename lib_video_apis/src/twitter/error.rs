use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct TwitterAPIError(pub String);

impl std::error::Error for TwitterAPIError{}

impl Display for TwitterAPIError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Twitter Api error: {}", self.0)
    }
}