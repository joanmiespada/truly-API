
use async_trait::async_trait;
use uuid::Uuid;
use crate::users::errors::users::UserNoExistsError;
use crate::users::models::user::{User, UserRoles, Userer};
use crate::users::repositories::users::{UsersRepo, UserRepository};

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error>>;



#[async_trait]
pub trait UserManipulation {
    async fn get_all(&self, page_number:u32, page_size:u32) -> ResultE<Vec<User>>;
    async fn get_by_user_id(&self,id:&String) -> ResultE<Option<User>>;
    async fn get_by_user_device(&self,device:&String) -> ResultE<User>;
    async fn get_by_user_email_and_password(&self, email:&String, password: &String) -> ResultE<User>;
    async fn add_user(&self, user:&mut User, password: &String) -> ResultE<String>;
    async fn get_by_filter(&self, field: &String, value: &String) -> ResultE<Vec<User>>;
    async fn update_user(&self,id:&String, user: &User) ->ResultE<bool> ;
    async fn promote_user_to_admin(&self, id: &String) ->ResultE<bool>;
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



#[async_trait]
impl UserManipulation for UsersService{

    async fn get_all(&self, page_number:u32, page_size:u32) -> ResultE<Vec<User>>{
        let res = self.repository.get_all(page_number, page_size).await?;
        Ok(res) 
    }

    async fn get_by_user_id(&self, id:&String) -> ResultE<Option<User>>{
        let res = self.repository.get_by_user_id(id).await?;
        Ok(res)
    }
    
    async fn get_by_user_device(&self,device:&String) -> ResultE<User>{
        let res = self.repository.get_by_user_device(device).await?;
        Ok(res)
    }
    async fn get_by_user_email_and_password(&self, email:&String, password: &String) -> ResultE<User>{
        let res = self.repository.get_by_user_email_and_password(email, password).await?;
        Ok(res)
    }


    async fn add_user(&self, user:&mut User, password: &String) -> ResultE<String>{
        let id = Uuid::new_v4();
        user.set_user_id(&id.to_string());
        user.roles_add(&UserRoles::Basic);
        let res =self.repository.add_user(user, password).await ?;
        Ok(res)
    }
    
    async fn get_by_filter(&self, field: &String, value: &String) -> ResultE<Vec<User>>{
        let res = self.repository.get_by_filter(field, value).await ?;
        Ok(res)
    }

    async fn update_user(&self, id: &String, user: &User) ->ResultE<bool> {
        let data =self.repository.get_by_user_id(id).await?; 
        match data {
           None =>  Err(UserNoExistsError("not found".to_string()).into()),
           Some(dbuser)=> {
                let mut res :User = dbuser.clone();

                res.set_email(user.email());
                res.set_wallet_address(user.wallet_address());
                res.set_device(user.device());

                let res = self.repository.update_user(&id, &res).await ?;
                Ok(res)
           },
        }
    }

    //Todo check if the thread's user is Admin, if not, this operation is fobidden
    async fn promote_user_to_admin(&self, id: &String) ->ResultE<bool>{

        let data =self.repository.get_by_user_id(id).await?; 
        match data {
           None =>  Err(UserNoExistsError("not found".to_string()).into()),
           Some(dbuser)=> {
                let mut res :User = dbuser.clone();
                res.promote_to_admin();
                let res = self.repository.update_user(&id, &res).await ?;
                Ok(res)
           },
        }
    }

}

impl Clone for UsersService{
    fn clone(&self) -> UsersService {
        let aux = UsersService{ repository: self.repository.clone()};
        return aux;
    }
}
