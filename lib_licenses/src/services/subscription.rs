use crate::errors::subscription::SubscriptionError;
use crate::models::subscription::{ConfirmedStatus, Subscription};
use crate::repositories::subscription::{SubscriptionRepo, SubscriptionRepository};
use lib_config::result::ResultE;
use uuid::Uuid;

pub const SERVICE: &str= "subscriptions";

pub struct SubscriptionService {
    subscription_repo: SubscriptionRepo,
}

impl SubscriptionService {
    pub fn new(subscription_repo: SubscriptionRepo) -> Self {
        Self { subscription_repo }
    }

    pub async fn find_user_subscriptions(&self, user_id: String) -> ResultE<Vec<Uuid>> {
        self.subscription_repo.find_by_user(user_id).await
    }
    
    pub async fn find_asset_subscriptions(&self, asset_id: Uuid) -> ResultE<Vec<String>> {
        self.subscription_repo.find_by_asset(asset_id).await
    }

    // pub async fn find_asset_subscriptors(&self, asset_id: Uuid) -> ResultE<Vec<String>> {
    //     self.subscription_repo.find_by_asset(asset_id).await
    // }

    pub async fn intent(&self, user_id: String, asset_id: Uuid) -> ResultE<Uuid> {
        let aux = self
            .subscription_repo
            .check_exists(user_id.clone(), asset_id)
            .await?;

        if let Some(subs) = aux {
            return Ok(subs);
        } else {
            let subscription = Subscription::new(user_id, asset_id, ConfirmedStatus::Disabled);
            self.subscription_repo.add(subscription).await
        }
    }

    pub async fn get(&self, id: Uuid) -> ResultE<Option<Subscription>> {
        self.subscription_repo.get_by_id(id).await
    }

    pub async fn delete(&self, id: Uuid) -> ResultE<()> {
        self.subscription_repo.delete(id).await
    }

    pub async fn confirm(&self, id: Uuid) -> ResultE<()> {
        let aux = self.subscription_repo.get_by_id(id).await?;

        match aux {
            None => return Err(Box::new(SubscriptionError::SubscriptionIDNotFound(id))),
            Some(mut subs) => {
                if subs.confirmed == ConfirmedStatus::Enabled {
                    return Ok(());
                }

                subs.confirmed = ConfirmedStatus::Enabled;
                self.subscription_repo.update(subs).await
            }
        }
    }

    pub async fn check_if_exists(&self, user_id: String, asset_id: Uuid) -> ResultE<Option<Uuid>> {
        self.subscription_repo.check_exists(user_id, asset_id).await
    }
}
