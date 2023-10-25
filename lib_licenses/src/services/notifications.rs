use crate::{models::notification::Notification, repositories::notifications::NotificationRepository};
use uuid::Uuid;
use lib_config::result::ResultE;

pub struct NotificationService<T: NotificationRepository> {
    repo: T,
}

impl<T: NotificationRepository> NotificationService<T> {
    pub fn new(repo: T) -> Self {
        NotificationService { repo }
    }

    pub async fn add(&self, notification: &Notification) -> ResultE<()> {
        self.repo.add(notification).await
    }

    pub async fn get(&self, notification_id: Uuid) -> ResultE<Option<Notification>> {
        self.repo.get(notification_id).await
    }

    pub async fn update(&self, notification: &Notification) -> ResultE<()> {
        self.repo.update(notification).await
    }

    pub async fn delete(&self, notification_id: Uuid) -> ResultE<()> {
        self.repo.delete(notification_id).await
    }

}
