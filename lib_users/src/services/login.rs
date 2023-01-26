use crate::errors::users::UserNoExistsError;
use crate::errors::users::UserStatusError;
use crate::models::user::UserRoles;
use async_trait::async_trait;
use tracing::instrument;

use super::users::UserManipulation;
use super::users::UsersService;

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error +Sync + Send >>;

pub struct LoginInfo {
    pub user_id: String,
    pub roles: Vec<UserRoles>,
}

#[async_trait]
pub trait LoginOps {
    async fn login(
        &self,
        device: &Option<String>,
        email: &Option<String>,
        passw: &Option<String>,
    ) -> ResultE<LoginInfo>;
}

#[async_trait]
impl LoginOps for UsersService {
    #[instrument]
    async fn login(
        &self,
        device: &Option<String>,
        email: &Option<String>,
        passw: &Option<String>,
    ) -> ResultE<LoginInfo> {
        let mut llt = LoginInfo {
            user_id: "".to_string(),
            roles: vec![],
        };
        let usr;
        if let Some(dvc) = device {
            usr = self.get_by_user_device(dvc).await?;
        } else if let Some(eml) = email {
            // && let Some(pwd) = passw {
            match passw {
                None => { return Err(UserNoExistsError("password is empty".to_string()).into());},
                Some(pss) => {
                    usr = self.get_by_user_email_and_password(eml, pss).await?;
                }
            }
        } else {
            return Err(UserNoExistsError("not correct parameters".to_string()).into());
        }

        if usr.status().is_disabled(){
            return Err(UserStatusError("user has been disabled".to_string()).into());
        } else {
            llt.user_id = usr.user_id().clone();
            llt.roles = usr.roles().clone();
            return Ok(llt);
        }
    }
}
