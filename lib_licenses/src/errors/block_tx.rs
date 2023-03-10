use std::fmt::Display;


#[derive(Debug)]
pub struct BlockchainTxError (pub String);

impl std::error::Error for BlockchainTxError {}

impl Display for BlockchainTxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "blockchain tx error: {}", self.0)
    }
}


#[derive(Debug)]
pub struct BlockchainTxNoExistsError(pub String);

impl std::error::Error for BlockchainTxNoExistsError{}

impl Display for BlockchainTxNoExistsError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "blockchain tx not found: {}", self.0)
    }
}