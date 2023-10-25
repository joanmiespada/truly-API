

#[derive(Debug, thiserror::Error )]
pub enum AlertSimilarError {
    #[error("Notification id already exists {0}")]
    AlertSimilarAlreadyExists (uuid::Uuid),

    #[error("Database error: {0}")]
    AlertSimilarDynamoDBError(#[from] Box<dyn std::error::Error + Sync + Send>),


    #[error("Notification not found with ID {0}")]
    AlertSimilarNotFound(uuid::Uuid),
}

