

#[derive(Debug)]
pub struct ApiLambdaAdminUserError(pub String);

impl std::error::Error for ApiLambdaAdminUserError{}

impl std::fmt::Display for ApiLambdaAdminUserError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "api lambda admin user error: {}", self.0)
    }
}