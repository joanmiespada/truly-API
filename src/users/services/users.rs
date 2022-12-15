use async_trait::async_trait;
use uuid::Uuid;
use crate::users::models::user::User;
use crate::users::repositories::users::{UsersRepo, UserRepository};

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait LoginOps {
    fn login() -> User;
}

#[async_trait]
pub trait UserManipulation {
    async fn get_all(&self, page_number:u32, page_size:u32) -> ResultE<Vec<User>>;
    async fn get_by_user_id(&self,id:String) -> ResultE<Option<User>>;
    async fn add_user(&self, user:&mut User) -> ResultE<String>;
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

    async fn get_all(&self, page_number:u32, page_size:u32) -> ResultE<Vec<User>>{
        let res = self.repository.get_all(page_number, page_size).await?;
        Ok(res) //.unwrap(); //vec![];
    }

    async fn get_by_user_id(&self, id:String) -> ResultE<Option<User>>{
        let res = self.repository.get_by_user_id(id).await?;
        Ok(res)//.unwrap(); //vec![];
        // return  None;
    }

    async fn add_user(&self, user:&mut User) -> ResultE<String>{
        let id = Uuid::new_v4();
        user.set_user_id(&id.to_string());
        let res =self.repository.add_user(user).await ?;
        Ok(res)
/* 
        let fin = match res {
            Some(newid) => Some(newid.to_string()),
            Err(e)=> Err(e)            
        };
        return fin;
        */
    }

}

impl Clone for UsersService{
    fn clone(&self) -> UsersService {
        let aux = UsersService{ repository: self.repository.clone()};
        return aux;
    }
}

