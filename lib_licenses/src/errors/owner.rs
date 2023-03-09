use std::fmt::Display;


#[derive(Debug)]
pub struct OwnerAlreadyExistsError(pub String);

impl std::error::Error for OwnerAlreadyExistsError {}

impl Display for OwnerAlreadyExistsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "owner already exists in database: {}", self.0)
    }
}

#[derive(Debug,Clone)]
pub struct OwnerDynamoDBError(pub String);


impl std::error::Error for OwnerDynamoDBError {}


impl Display for OwnerDynamoDBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "owner database error: {}", self.0)
    }
}

#[derive(Debug)]
pub struct OwnerNoExistsError(pub String);

impl std::error::Error for OwnerNoExistsError {}

impl Display for OwnerNoExistsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "owner doesn't exists in database: {}", self.0)
    }
}



