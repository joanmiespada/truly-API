

#[derive(Debug)]
pub struct ApiLambdaError(pub String);

impl std::error::Error for ApiLambdaError{}

impl std::fmt::Display for ApiLambdaError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "api lambda asset error: {}", self.0)
    }
}