use crate::errors::subscription::SubscriptionError;
use crate::models::subscription::{ConfirmedStatus, Subscription};
use crate::repositories::sender::SenderEmailsRepo;
use crate::repositories::subscription::SubscriptionRepository;
use lib_config::result::ResultE;
use lib_licenses::models::asset::Asset;
use lib_users::models::user::User;
use uuid::Uuid;


pub const SERVICE: &str= "subscriptions";

pub struct SubscriptionService<T: SubscriptionRepository> {
    subscription_repo: T ,
    sender_emails_repo: SenderEmailsRepo,
}

impl<T: SubscriptionRepository>  SubscriptionService<T> {
    pub fn new(subscription_repo: T, sender_emails_repo: SenderEmailsRepo) -> Self {
        SubscriptionService { subscription_repo, sender_emails_repo }
    }

    pub async fn find_assets_subscribed_to(&self, user_id: String) -> ResultE<Vec<Subscription>> {
        self.subscription_repo.find_by_user(user_id).await
    }
    
    pub async fn find_users_subscribed_to(&self, asset_id: Uuid) -> ResultE<Vec<Subscription>> {
        self.subscription_repo.find_by_asset(asset_id).await
    }

    // pub async fn find_asset_subscriptors(&self, asset_id: Uuid) -> ResultE<Vec<String>> {
    //     self.subscription_repo.find_by_asset(asset_id).await
    // }

    pub async fn intent(&self, user: User, asset: Asset) -> ResultE<Uuid> {
        let aux = self
            .subscription_repo
            .check_exists(user.user_id().clone(), asset.id().clone() )
            .await?;

        if let Some(subs) = aux {
            return Ok(subs);
        } else {
            let subscription = Subscription::new(user.user_id().clone(), asset.id().clone(), ConfirmedStatus::Disabled);
            let op =self.subscription_repo.add(subscription.clone()).await;

            if let Ok(id) = op {
                let mut subscription = subscription.clone();
                subscription.id = id;
                self.sender_emails_repo.send_intent(user, asset, subscription).await?;
            } 

            op
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
