use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct ContractDynamoDBError(pub String);

impl std::error::Error for ContractDynamoDBError {}

impl Display for ContractDynamoDBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "contract database error: {}", self.0)
    }
}

#[derive(Debug)]
pub struct ContractNoExistsError(pub String);

impl std::error::Error for ContractNoExistsError {}

impl Display for ContractNoExistsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "contract doesn't exists in database: {}", self.0)
    }
}
