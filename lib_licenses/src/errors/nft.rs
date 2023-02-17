use std::fmt::Display;

use uuid::Uuid;


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
        write!(f, "private key secret for asset owner error")
    }
}



#[derive(Debug)]
pub struct HydrateMasterSecretKeyError;

impl std::error::Error for HydrateMasterSecretKeyError{}

impl Display for HydrateMasterSecretKeyError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "master private key secret for contract owner error")
    }
}

#[derive(Debug)]
pub struct TokenHasBeenMintedAlreadyError(pub Uuid);

impl std::error::Error for TokenHasBeenMintedAlreadyError{}

impl Display for TokenHasBeenMintedAlreadyError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "this token has been already minted. You can't mint it twice. {}", self.0.to_string())
    }
}

#[derive(Debug)]
pub struct TokenMintingProcessHasBeenInitiatedError (pub Uuid);

impl std::error::Error for TokenMintingProcessHasBeenInitiatedError {}

impl Display for TokenMintingProcessHasBeenInitiatedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "this token has started the minting processa. Let's wait until current process finishes. token: {}", self.0.to_string())
    }
}