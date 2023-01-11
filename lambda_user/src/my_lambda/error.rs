

#[derive(Debug)]
pub struct ApiLambdaUserError(pub String);

impl std::error::Error for ApiLambdaUserError{}

impl std::fmt::Display for ApiLambdaUserError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "api lambda admin user error: {}", self.0)
    }
}