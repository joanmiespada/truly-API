

#[derive(Debug)]
pub struct ApiLambdaAssetError(pub String);

impl std::error::Error for ApiLambdaAssetError{}

impl std::fmt::Display for ApiLambdaAssetError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "api lambda asset error: {}", self.0)
    }
}