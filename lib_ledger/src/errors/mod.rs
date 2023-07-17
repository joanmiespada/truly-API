use std::fmt::Display;

#[derive(Debug)]
pub struct LedgerError(pub String);

impl std::error::Error for LedgerError {}

impl Display for LedgerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ledger error: {}", self.0)
    }
}

