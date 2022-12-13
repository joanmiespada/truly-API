use actix_web::Result;
use async_trait::async_trait;
use uuid::Uuid;
use crate::users::models::user::User;
use crate::users::repositories::users::{UsersRepo, UserRepository};


pub trait LoginOps {
    fn login() -> User;
}

#[async_trait]
pub trait UserManipulation {
    async fn get_all(&self, page_number:u32, page_size:u32) -> Vec<User>;
    async fn get_by_user_id(&self,id:String) -> Option<User>;
    async fn add_user(&self, user:&mut User) -> String;
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

    async fn add_user(&self, user:&mut User) -> String{
        let id = Uuid::new_v4();
        *user.set_user_id() = id.to_string();
        self.repository.add_user(user).await;
        return id.to_string();
    }

}

impl Clone for UsersService{
    fn clone(&self) -> UsersService {
        let aux = UsersService{ repository: self.repository.clone()};
        return aux;
    }
}

