
#[derive(Debug, thiserror::Error)]
pub enum SubscriptionError {
    #[error("Subscription already exists for user {user_id} with asset {asset_id}")]
    SubscriptionAlreadyExists {
        user_id: String,
        asset_id: uuid::Uuid,
    },

    #[error("Subscription not found for asset_id: {0} user_id: {1}")]
    SubscriptionNotFound(uuid::Uuid, String),

    #[error("Asset not found with ID {0}")]
    AssetNotFound(uuid::Uuid),

    #[error("User not found with ID {0}")]
    UserNotFound(String),

    #[error("Database error: {0}")]
    SubscriptionDynamoDBError(#[from] Box<dyn std::error::Error + Sync + Send>),


    #[error("Subscription not found with ID {0}")]
    SubscriptionIDNotFound(uuid::Uuid),
}
