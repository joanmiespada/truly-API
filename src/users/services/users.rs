use async_trait::async_trait;

use crate::users::models::user::User;
use crate::users::repositories::users::{UsersRepo, UserRepository};


pub trait LoginOps {
    fn login() -> User;
}

#[async_trait]
pub trait UserManipulation {
    async fn get_all(&self, page_number:u32, page_size:u32) -> Vec<User>;
    async fn get_by_user_id(&self,id:String) -> Option<User>;
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

    fn login() -> User {
        let user = User::new();
        return user;
    }
}

#[async_trait]
impl UserManipulation for UsersService{

    async fn get_all(&self, page_number:u32, page_size:u32) -> Vec<User>{
        let res = self.repository.get_all(page_number, page_size).await;
        return  res.unwrap(); //vec![];
    }

    async fn get_by_user_id(&self, id:String) -> Option<User>{
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

