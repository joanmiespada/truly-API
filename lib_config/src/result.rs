
pub type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;