use std::fmt::Display;


#[derive(Debug)]
pub struct VideoError(pub String);

impl std::error::Error for VideoError{}

impl Display for VideoError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "video error: {}", self.0)
    }
}