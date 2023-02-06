use std::fmt::Display;


#[derive(Debug)]
pub struct NftUserAddressMalformedError(pub String);

impl std::error::Error for NftUserAddressMalformedError{}

impl Display for NftUserAddressMalformedError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "user wallet address incorrect: {}", self.0)
    }
}

#[derive(Debug)]
pub struct NftBlockChainNonceMalformedError(pub String);

impl std::error::Error for NftBlockChainNonceMalformedError{}

impl Display for NftBlockChainNonceMalformedError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "nonce generator error: {}", self.0)
    }
}

#[derive(Debug)]
pub struct NftBlockChainSecretOwnerMalformedError;

impl std::error::Error for NftBlockChainSecretOwnerMalformedError{}

impl Display for NftBlockChainSecretOwnerMalformedError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "private key secret for owner error")
    }
}

