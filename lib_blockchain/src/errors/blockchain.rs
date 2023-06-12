use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct BlockchainDynamoDBError(pub String);

impl std::error::Error for BlockchainDynamoDBError {}

impl Display for BlockchainDynamoDBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "blockchain database error: {}", self.0)
    }
}

#[derive(Debug)]
pub struct BlockchainNoExistsError(pub String);

impl std::error::Error for BlockchainNoExistsError {}

impl Display for BlockchainNoExistsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "blockchain doesn't exists in database: {}", self.0)
    }
}
#[derive(Debug)]
pub struct BlockchainKeyPairGenError(pub String);

impl std::error::Error for BlockchainKeyPairGenError {}

impl Display for BlockchainKeyPairGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "blockchain keypair generation error: {}", self.0)
    }
}
