use crate::config::EnvironmentVariables;
use crate::users::errors::users::UserNoExistsError;
use crate::users::models::user::{User, UserRoles, UserStatus, Userer};
use crate::users::repositories::users::{UserRepository, UsersRepo};
use async_trait::async_trait;
use uuid::Uuid;

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[async_trait]
pub trait UserManipulation {
    async fn get_all(&self, page_number: u32, page_size: u32) -> ResultE<Vec<User>>;
    async fn get_by_user_id(&self, id: &String) -> ResultE<User>;
    async fn get_by_user_device(&self, device: &String) -> ResultE<User>;
    async fn get_by_user_email_and_password(
        &self,
        email: &String,
        password: &String,
    ) -> ResultE<User>;
    async fn add_user(&self, user: &mut User, password: &Option<String>) -> ResultE<String>;
    //async fn get_by_filter(&self, field: &String, value: &String) -> ResultE<Vec<User>>;
    async fn update_user(&self, id: &String, user: &UpdatableFildsUser) -> ResultE<bool>;
    async fn promote_user_to(&self, id: &String, promo: &promote_user) -> ResultE<bool>;
    async fn update_password(&self, id: &String, password: &String) -> ResultE<()>;
}

#[derive(Debug)]
pub struct UsersService {
    repository: UsersRepo,
}

impl UsersService {
    pub fn new(repo: UsersRepo) -> UsersService {
        UsersService { repository: repo }
    }
}
pub enum promote_user {
    downgrade,
    upgrade,
}
#[derive(Debug)]
pub struct UpdatableFildsUser {
    pub email: Option<String>,
    pub wallet_address: Option<String>,
    pub device: Option<String>,
    pub status: Option<String>,
}
impl UpdatableFildsUser {
    pub fn new() -> UpdatableFildsUser {
        UpdatableFildsUser {
            email: None,
            wallet_address: None,
            device: None,
            status: None,
        }
    }
}

#[async_trait]
impl UserManipulation for UsersService {
    #[tracing::instrument()]
    async fn get_all(&self, page_number: u32, page_size: u32) -> ResultE<Vec<User>> {
        let res = self.repository.get_all(page_number, page_size).await?;
        Ok(res)
    }

    #[tracing::instrument()]
    async fn get_by_user_id(&self, id: &String) -> ResultE<User> {
        let res = self.repository.get_by_user_id(id).await?;
        Ok(res)
    }

    #[tracing::instrument( fields( device= tracing::field::Empty) )]
    async fn get_by_user_device(&self, device: &String) -> ResultE<User> {
        tracing::Span::current().record("device", &tracing::field::display(&device));
        let res = self.repository.get_by_user_device(device).await?;
        Ok(res)
    }
    #[tracing::instrument(fields(email, success = false))]
    async fn get_by_user_email_and_password(
        &self,
        email: &String,
        password: &String,
    ) -> ResultE<User> {
        tracing::Span::current().record("email", &tracing::field::display(&email));

        let res = self
            .repository
            .get_by_user_email_and_password(email, password)
            .await?;

        tracing::Span::current().record("success", &tracing::field::display(true));
        Ok(res)
    }

    #[tracing::instrument()]
    async fn add_user(&self, user: &mut User, password: &Option<String>) -> ResultE<String> {
        let id = Uuid::new_v4();
        user.set_user_id(&id.to_string());
        user.roles_add(&UserRoles::Basic);
        let res = self.repository.add_user(user, password).await?;
        Ok(res)
    }

    /*async fn get_by_filter(&self, field: &String, value: &String) -> ResultE<Vec<User>> {
        let res = self.repository.get_by_filter(field, value).await?;
        Ok(res)
    }*/

    #[tracing::instrument()]
    async fn update_user(&self, id: &String, user: &UpdatableFildsUser) -> ResultE<bool> {
        let dbuser = self.repository.get_by_user_id(id).await?;
        let mut res: User = dbuser.clone();

        match &user.email {
            None => (),
            Some(eml) => res.set_email(&eml),
        }
        match &user.wallet_address {
            None => (),
            Some(wa) => {
                res.set_wallet_address(&wa);
            }
        }
        match &user.device {
            None => (),
            Some(dvc) => res.set_device(&dvc),
        }
        match &user.status {
            None => (),
            Some(sts) => {
                let aux = UserStatus::parse(&sts);
                match aux {
                    None => {}
                    Some(sts_val) => res.set_status(&sts_val),
                }
            }
        }

        let res = self.repository.update_user(&id, &res).await?;
        Ok(res)
    }

    #[tracing::instrument()]
    async fn update_password(&self, id: &String, password: &String) -> ResultE<()> {
        _ = self.repository.update_password(id, password).await?;
        Ok(())
    }

    async fn promote_user_to(&self, id: &String, promo: &promote_user) -> ResultE<bool> {
        let dbuser = self.repository.get_by_user_id(id).await?;
        let mut res: User = dbuser.clone();
        match promo {
            promote_user::upgrade => {
                res.promote_to_admin();
            }
            promote_user::downgrade => {
                res.downgrade_from_admin();
            }
        }

        let res = self.repository.update_user(&id, &res).await?;
        Ok(res)
    }
}

impl Clone for UsersService {
    #[tracing::instrument()]
    fn clone(&self) -> UsersService {
        let aux = UsersService {
            repository: self.repository.clone(),
        };
        return aux;
    }
}
