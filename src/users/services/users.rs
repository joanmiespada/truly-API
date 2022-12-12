use async_trait::async_trait;

use crate::users::models::user::User;
use crate::users::repositories::users::{UsersRepo, UserRepository};


pub trait LoginOps {
    fn Login() -> User;
}

#[async_trait]
pub trait UserManipulation {
    async fn get_all(&self) -> Vec<User>;
    async fn get_by_user_id(&self,id:i64) -> Option<User>;
}

pub struct UsersService{
    repository: UsersRepo
}

impl UsersService {
    pub fn new(repo: UsersRepo) -> UsersService {
       UsersService {
        repository: repo
       } 
    }
}

impl LoginOps for UsersService {

    fn Login() -> User {
        let user = User::new();
        return user;
    }
}

#[async_trait]
impl UserManipulation for UsersService{

    async fn get_all(&self) -> Vec<User>{
        let res = self.repository.get_all().await;
        return  res.unwrap(); //vec![];
    }

    async fn get_by_user_id(&self, id:i64) -> Option<User>{
        let res = self.repository.get_by_user_id(id).await;
        return  res.unwrap(); //vec![];
        // return  None;
    }

}

impl Clone for UsersService{
    fn clone(&self) -> UsersService {
        let aux = UsersService{ repository: self.repository.clone()};
        return aux;
    }
}

