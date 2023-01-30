use std::fmt::Display;


#[derive(Debug)]
pub struct NftUserAddressMalformedError(pub String);

impl std::error::Error for NftUserAddressMalformedError{}

impl Display for NftUserAddressMalformedError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "user wallet address incorrect: {}", self.0)
    }
}

