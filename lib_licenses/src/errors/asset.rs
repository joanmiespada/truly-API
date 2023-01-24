use std::fmt::Display;


#[derive(Debug)]
pub struct AssetAlreadyExistsError(pub String);

impl std::error::Error for AssetAlreadyExistsError {}

impl Display for AssetAlreadyExistsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "asset already exists in database: {}", self.0)
    }
}

#[derive(Debug,Clone)]
pub struct AssetDynamoDBError(pub String);


impl std::error::Error for AssetDynamoDBError {}


impl Display for AssetDynamoDBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "asset database error: {}", self.0)
    }
}

#[derive(Debug)]
pub struct AssetNoExistsError(pub String);

impl std::error::Error for AssetNoExistsError {}

impl Display for AssetNoExistsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "asset doesn't exists in database: {}", self.0)
    }
}

#[derive(Debug)]
pub struct AssetMismatchError(pub String);

impl std::error::Error for AssetMismatchError {}

impl Display for AssetMismatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "asset's key data might repeated in database: {}", self.0)
    }
}

#[derive(Debug)]
pub struct AssetParamNotAccepted(pub String);

impl std::error::Error for AssetParamNotAccepted{}

impl Display for AssetParamNotAccepted{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "asset's data is unacceptable: {}", self.0)
    }
}

#[derive(Debug)]
pub struct AssetStatusError(pub String);

impl std::error::Error for AssetStatusError{}

impl Display for AssetStatusError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "asset's status: {}", self.0)
    }
}

#[derive(Debug)]
pub struct AssetBlockachainError(pub String);

impl std::error::Error for AssetBlockachainError{}

impl Display for AssetBlockachainError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "blockchain error: {}", self.0)
    }
}