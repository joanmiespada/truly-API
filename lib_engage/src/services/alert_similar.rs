use std::time::{SystemTime, Duration};

use crate::{models::alert_similar::{AlertSimilar, AlertSimilarBuilder}, repositories::alert_similar::AlertSimilarRepository};
use chrono::Utc;
use uuid::Uuid;
use lib_config::result::ResultE;

pub struct AlertSimilarService<T: AlertSimilarRepository> {
    repo: T,
}

impl<T: AlertSimilarRepository> AlertSimilarService<T> {
    pub fn new(repo: T) -> Self {
        AlertSimilarService { repo }
    }

    pub async fn add(&self, notification_builder: & mut AlertSimilarBuilder) -> ResultE<AlertSimilar> {
        let new_id = uuid::Uuid::new_v4();
        //let mut notification = notification_builder.copy();
        let notification = notification_builder
            .id(new_id)
            .creation_time(Utc::now())
            .last_update_time(Utc::now())
            .build()?;
        self.repo.add(&notification).await?;
        Ok(notification)
    }

    pub async fn get(&self, notification_id: Uuid) -> ResultE<Option<AlertSimilar>> {
        self.repo.get(notification_id).await
    }

    pub async fn update(&self, notification: &AlertSimilar) -> ResultE<()> {
        self.repo.update(notification).await
    }

    pub async fn delete(&self, notification_id: Uuid) -> ResultE<()> {
        self.repo.delete(notification_id).await
    }
    pub async fn get_latests(&self, 
        starting_at: SystemTime, 
        window:Duration, 
        token: Option<String>, 
        page_size: Option<i32>  ) -> ResultE<(Vec<AlertSimilar>,Option<String>)> {

        self.repo.get_all_by_time( starting_at, window, token, page_size).await
    }
    pub async fn get_all(&self,  
        token: Option<String>, 
        page_size: Option<i32>  ) -> ResultE<(Vec<AlertSimilar>,Option<String>)> {

        self.repo.get_all(token, page_size).await
    }

}
