use std::fmt::Display;

#[derive(Debug)]
pub struct KeyPairAlreadyExistsError(pub String);

impl std::error::Error for KeyPairAlreadyExistsError {}

impl Display for KeyPairAlreadyExistsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KeyPair already exists in database: {}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct KeyPairDynamoDBError(pub String);

impl std::error::Error for KeyPairDynamoDBError {}

impl Display for KeyPairDynamoDBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "keypair database error: {}", self.0)
    }
}

#[derive(Debug)]
pub struct KeyPairNoExistsError(pub String);

impl std::error::Error for KeyPairNoExistsError {}

impl Display for KeyPairNoExistsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "keypair doesn't exists in database: {}", self.0)
    }
}
