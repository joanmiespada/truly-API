use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct VimeoAPIError(pub String);

impl std::error::Error for VimeoAPIError{}

impl Display for VimeoAPIError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VimeoApi error: {}", self.0)
    }
}