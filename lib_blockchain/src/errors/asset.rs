use std::fmt::Display;

#[derive(Debug)]
pub struct AssetBlockachainError(pub String);

impl std::error::Error for AssetBlockachainError {}

impl Display for AssetBlockachainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "blockchain error: {}", self.0)
    }
}

#[derive(Debug)]
pub struct AssetNoExistsError(pub String);

impl std::error::Error for AssetNoExistsError {}

impl Display for AssetNoExistsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "asset not found at tx: {}", self.0)
    }
}