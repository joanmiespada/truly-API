

#[derive(Debug, thiserror::Error )]
pub enum NotificationError {
    #[error("Notification id already exists {0}")]
    NotificationAlreadyExists (uuid::Uuid),

    #[error("Database error: {0}")]
    NotificationDynamoDBError(#[from] Box<dyn std::error::Error + Sync + Send>),


    #[error("Notification not found with ID {0}")]
    NotificationNotFound(uuid::Uuid),
}

