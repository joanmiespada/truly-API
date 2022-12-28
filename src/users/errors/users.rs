use std::fmt::Display;


#[derive(Debug)]
pub struct UserAlreadyExistsError(pub String);

impl std::error::Error for UserAlreadyExistsError {}

impl Display for UserAlreadyExistsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "user already exists in database: {}", self.0)
    }
}

#[derive(Debug,Clone)]
pub struct DynamoDBError(pub String);


impl std::error::Error for DynamoDBError {}


impl Display for DynamoDBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "database error: {}", self.0)
    }
}

#[derive(Debug)]
pub struct UserNoExistsError(pub String);

impl std::error::Error for UserNoExistsError {}

impl Display for UserNoExistsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "user doesn't exists in database: {}", self.0)
    }
}

#[derive(Debug)]
pub struct UserMismatchError(pub String);

impl std::error::Error for UserMismatchError {}

impl Display for UserMismatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "user's key data might repeated in database: {}", self.0)
    }
}
#[derive(Debug)]
pub struct UserPasswordError(pub String);

impl std::error::Error for UserPasswordError {}

impl Display for UserPasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "user's password check: {}", self.0)
    }
}
#[derive(Debug)]
pub struct UserParamNotAccepted(pub String);

impl std::error::Error for UserParamNotAccepted{}

impl Display for UserParamNotAccepted{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "user's data is unacceptable: {}", self.0)
    }
}
