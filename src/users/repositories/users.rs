use std::io::Error;

use async_trait::async_trait;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{Client};

use crate::users::models::user::User;

#[async_trait]
pub trait UserRepository {
    async fn configure(&mut self) -> Result<(), Error>;
    async fn add_user(&self,user: &User) -> Result<(), Error>;
    async fn get_by_user_id(&self, id: i64) -> Result<Option<User>, Error>;
    async fn get_all(&self) -> Result<Vec<User>, Error> ;
}

pub struct UsersRepo {
    client: Option<Client>,
}

impl UsersRepo{
    pub fn new() -> UsersRepo {
        UsersRepo { client: None }
    }
}

impl Clone for UsersRepo{
    fn clone(&self) -> UsersRepo {
        let mut aux = UsersRepo{ client: None };
        aux.configure();
        return aux;
    }
}


#[async_trait]
impl UserRepository for UsersRepo {
    async fn configure(&mut self) -> Result<(), Error> {
        //let shared_config = aws_config::load_from_env().await;
        //client = Client::new(&shared_config);

        let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
        let config = aws_config::from_env().region(region_provider).load().await;
        self.client = Some( Client::new(&config));

        Ok(())

    }
 
    async fn add_user(&self, user: &User) -> Result<(), Error> {
/*         let user_av = AttributeValue::S(item.username);
        let type_av = AttributeValue::S(item.p_type);
        let age_av = AttributeValue::S(item.age);
        let first_av = AttributeValue::S(item.first);
        let last_av = AttributeValue::S(item.last);

        let request = client
            .put_item()
            .table_name(table)
            .item("username", user_av)
            .item("account_type", type_av)
            .item("age", age_av)
            .item("first_name", first_av)
            .item("last_name", last_av);

        println!("Executing request [{request:?}] to add item...");

        let resp = request.send().await?;

        let attributes = resp.attributes().unwrap();

        println!(
            "Added user {:?}, {:?} {:?}, age {:?} as {:?} user",
            attributes.get("username"),
            attributes.get("first_name"),
            attributes.get("last_name"),
            attributes.get("age"),
            attributes.get("p_type")
        );
        */
        Ok(())
    }
    
    async fn get_all(&self) -> Result<Vec<User>, Error> {
        let mut aux = Vec::new();
        aux.push( User::new()  );

        Ok(aux)
    }

    async fn get_by_user_id(&self, id: i64) -> Result<Option<User>, Error> {
        Ok( Some(User::new())  ) 
    }
}

/* 
pub fn CreateUserRepo() -> UsersRepo {
    let aux = UsersRepo { client: todo!()  };

    aux.configure();

    return aux;
        
}
*/