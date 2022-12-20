use std::collections::HashMap;

use async_trait::async_trait;

use crate::users::errors::users::UserNoExistsError;
use crate::users::models::user::{User, UserRoles, Userer};
use crate::users::repositories::users::{UserRepository, UsersRepo};

use super::users::UserManipulation;
use super::users::UsersService;

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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

        if let Some(dvc) = device {
            let usr = self.get_by_user_device(dvc).await?;
            llt.user_id = usr.user_id().clone();
            llt.roles = usr.roles().clone();
            return Ok(llt);
        } else if let Some(eml) = email {
            // && let Some(pwd) = passw {
            match passw {
                None => Err(UserNoExistsError("password is empty".to_string()).into()),
                Some(pss) => {
                    let usr = self.get_by_user_email_and_password(eml, pss).await?;
                    llt.user_id = usr.user_id().clone();
                    llt.roles = usr.roles().clone();
                    return Ok(llt);
                }
            }
        } else {
            return Err(UserNoExistsError("not correct parameters".to_string()).into());
        }
    }
}
