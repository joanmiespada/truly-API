use crate::models::user::{User, UserRoles, UserStatus, Userer};
use crate::repositories::users::{UserRepository, UsersRepo};
use crate::validate_password;
use async_trait::async_trait;
use uuid::Uuid;

use validator::Validate;
type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error +Sync + Send >>;

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
    async fn promote_user_to(&self, id: &String, promo: &PromoteUser) -> ResultE<bool>;
    async fn update_password(&self, id: &String, password: &String) -> ResultE<()>;
    async fn remove_by_id(&self, user_id: &String) -> ResultE<()>;
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
#[derive(Debug)]
pub enum PromoteUser {
    Downgrade,
    Upgrade,
}
#[derive(Debug,Validate)]
pub struct UpdatableFildsUser {
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(max=100))]
    pub device: Option<String>,
    #[validate(length(max=10))]
    pub status: Option<String>,
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

        match password {
            None=> {},
            Some(pass) => validate_password(pass)?
        }

        let id = Uuid::new_v4();
        user.set_user_id(&id.to_string());
        user.roles_add(&UserRoles::Basic);
        user.validate()?;
        let res = self.repository.add_user(user, password).await?;
        Ok(res)
    }
    
    #[tracing::instrument()]
    async fn remove_by_id(&self, id: &String) -> ResultE<()> {
        let user = self.get_by_user_id(id).await?;
        let res = self.repository.remove(user.user_id()).await?;
        Ok(res)
    }

    #[tracing::instrument()]
    async fn update_user(&self, id: &String, user: &UpdatableFildsUser) -> ResultE<bool> {
        
        user.validate()?;
        
        let dbuser = self.repository.get_by_user_id(id).await?;
        let mut res: User = dbuser.clone();


        match &user.email {
            None => (),
            Some(eml) => res.set_email(&eml),
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

        validate_password(password)?;
        _ = self.repository.update_password(id, password).await?;
        Ok(())
    }

    async fn promote_user_to(&self, id: &String, promo: &PromoteUser) -> ResultE<bool> {
        let dbuser = self.repository.get_by_user_id(id).await?;
        let mut res: User = dbuser.clone();
        match promo {
            PromoteUser::Upgrade => {
                res.promote_to_admin();
            }
            PromoteUser::Downgrade => {
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
