#[derive(Debug)]
pub struct LicenseNotFoundError(pub String);

impl std::error::Error for LicenseNotFoundError {}

impl std::fmt::Display for LicenseNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "License not found: {}", self.0)
    }
}

#[derive(Debug)]
pub struct LicenseCreationError(pub String);

impl std::error::Error for LicenseCreationError {}

impl std::fmt::Display for LicenseCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to create license: {}", self.0)
    }
}

#[derive(Debug)]
pub struct LicenseDynamoDBError(pub String);

impl std::error::Error for LicenseDynamoDBError {}

impl std::fmt::Display for LicenseDynamoDBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "License DynamoDB error: {}", self.0)
    }
}
