use std::fmt::Display;


#[derive(Debug)]
pub struct UserAlreadyExistsError(pub String);

impl std::error::Error for UserAlreadyExistsError {}

impl Display for UserAlreadyExistsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "user already exists in database")
    }
}

#[derive(Debug,Clone)]
pub struct DynamoDBError(pub String);


impl std::error::Error for DynamoDBError {}


impl Display for DynamoDBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}