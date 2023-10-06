use crate::models::user::{User, UserRoles, UserStatus, Userer};
use crate::repositories::users::{UserRepository, UsersRepo};
use crate::validate_password;
use async_trait::async_trait;
use uuid::Uuid;

use validator::Validate;
type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[async_trait]
pub trait UserManipulation {
    async fn get_all(&self, page_number: u32, page_size: u32) -> ResultE<Vec<User>>;
    async fn get_by_id(&self, id: &String) -> ResultE<User>;
    async fn get_by_device(&self, device: &String) -> ResultE<User>;
    async fn get_by_wallet(&self, wallet_address: &String) -> ResultE<User>;
    async fn get_by_email_and_password(&self, email: &String, password: &String) -> ResultE<User>;
    async fn add(&self, user: &mut User, password: &Option<String>) -> ResultE<String>;
    //async fn get_by_filter(&self, field: &String, value: &String) -> ResultE<Vec<User>>;
    async fn update(&self, id: &String, user: &UpdatableFildsUser) -> ResultE<()>;
    async fn promote_user_to(&self, id: &String, promo: &PromoteUser) -> ResultE<()>;
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
#[derive(Debug, Validate)]
pub struct UpdatableFildsUser {
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(max = 100))]
    pub device: Option<String>,
    #[validate(length(max = 10))]
    pub status: Option<String>,
    #[validate(length(max = 100))]
    pub wallet: Option<String>,
}

#[async_trait]
impl UserManipulation for UsersService {
    //#[tracing::instrument()]
    async fn get_all(&self, page_number: u32, page_size: u32) -> ResultE<Vec<User>> {
        let res = self.repository.get_all(page_number, page_size).await?;
        Ok(res)
    }

    //#[tracing::instrument()]
    async fn get_by_id(&self, id: &String) -> ResultE<User> {
        let res = self.repository.get_by_id(id).await?;
        Ok(res)
    }

    //#[tracing::instrument( fields( device= tracing::field::Empty) )]
    async fn get_by_device(&self, device: &String) -> ResultE<User> {
        //tracing::Span::current().record("device", &tracing::field::display(&device));
        let res = self.repository.get_by_device(device).await?;
        Ok(res)
    }
    async fn get_by_wallet(&self, wallet_address: &String) -> ResultE<User> {
        //tracing::Span::current()
            //.record("wallet_address", &tracing::field::display(&wallet_address));
        let res = self
            .repository
            .get_by_wallet_address(wallet_address)
            .await?;
        Ok(res)
    }
    //#[tracing::instrument(fields(email, success = false), skip(password))]
    async fn get_by_email_and_password(&self, email: &String, password: &String) -> ResultE<User> {
        //tracing::Span::current().record("email", &tracing::field::display(&email));

        let res = self
            .repository
            .get_by_email_and_password(email, password)
            .await?;

        //tracing::Span::current().record("success", &tracing::field::display(true));
        Ok(res)
    }

    //#[tracing::instrument()]
    async fn add(&self, user: &mut User, password: &Option<String>) -> ResultE<String> {
        match password {
            None => {}
            Some(pass) => validate_password(pass)?,
        }

        let id = Uuid::new_v4();
        user.set_user_id(&id.to_string());
        if user.roles().len() == 0 {
            user.roles_add(&UserRoles::Basic);
        }
        user.validate()?;
        self.repository.add(user, password).await?;
        Ok(id.to_string())
    }

    //#[tracing::instrument()]
    async fn remove_by_id(&self, id: &String) -> ResultE<()> {
        let user = self.get_by_id(id).await?;
        let res = self.repository.remove(user.user_id()).await?;
        Ok(res)
    }

    //#[tracing::instrument()]
    async fn update(&self, id: &String, user: &UpdatableFildsUser) -> ResultE<()> {
        user.validate()?;

        let dbuser = self.repository.get_by_id(id).await?;
        let mut user_changes: User = dbuser.clone();

        if let Some(wal) = &user.wallet {
            user_changes.set_wallet_address(&wal);
        }
        if let Some(eml) = &user.email {
            user_changes.set_email(&eml);
        }
        if let Some(dvc) = &user.device {
            user_changes.set_device(&dvc);
        }
        if let Some(sts) = &user.status {
            let aux = UserStatus::parse(&sts);
            if let Some(sts_val) = aux {
                user_changes.set_status(&sts_val);
            }
        }

        self.repository.update(&id, &user_changes).await?;
        Ok(())
    }

    //#[tracing::instrument()]
    async fn update_password(&self, id: &String, password: &String) -> ResultE<()> {
        validate_password(password)?;
        _ = self.repository.update_password(id, password).await?;
        Ok(())
    }

    async fn promote_user_to(&self, id: &String, promo: &PromoteUser) -> ResultE<()> {
        let dbuser = self.repository.get_by_id(id).await?;
        let mut res: User = dbuser.clone();
        match promo {
            PromoteUser::Upgrade => {
                res.promote_to_admin();
            }
            PromoteUser::Downgrade => {
                res.downgrade_from_admin();
            }
        }

        self.repository.update(&id, &res).await?;
        Ok(())
    }
}

impl Clone for UsersService {
    //#[tracing::instrument()]
    fn clone(&self) -> UsersService {
        let aux = UsersService {
            repository: self.repository.clone(),
        };
        return aux;
    }
}
