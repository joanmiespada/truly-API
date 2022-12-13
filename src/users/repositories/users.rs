use std::io::Error;

use crate::users::models::user::User;
use async_trait::async_trait;
use aws_config::{meta::region::RegionProviderChain, SdkConfig};
use aws_sdk_dynamodb::{model::{AttributeValue, Select}, Client};
use chrono::{
    prelude::{DateTime, Utc},
    Local,
};

#[async_trait]
pub trait UserRepository {
    //async fn configure(&mut self) -> Result<(), Error>;
    async fn add_user(&self, user: &mut User) -> Result<(), Error>;
    async fn get_by_user_id(&self, id: String) -> Result<Option<User>, Error>;
    async fn get_all(&self, page_number: u32, page_size: u32) -> Result<Vec<User>, Error>;
}

pub struct UsersRepo {
    client: Client,
}

impl UsersRepo {
    pub fn new(aux: &SdkConfig) -> UsersRepo {
        UsersRepo {
            client: Client::new(aux),
        }
    }
}

impl Clone for UsersRepo {
    fn clone(&self) -> UsersRepo {
        let aux = UsersRepo {
            client: self.client.clone(),
        };
        //aux.configure();
        return aux;
    }
}

#[async_trait]
impl UserRepository for UsersRepo {
    /*
    async fn configure(&mut self) -> Result<(), Error> {
        //let shared_config = aws_config::load_from_env().await;
        //client = Client::new(&shared_config);

        //let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
        //let config = aws_config::from_env().region(region_provider).load().await;
        self.client = Some( Client::new(&config));

        Ok(())

    }*/

    async fn add_user(&self, user: &mut User) -> Result<(), Error> {
        let user_id_av = AttributeValue::S(user.get_user_id().clone());
        let device_av = AttributeValue::S(user.get_device().clone());
        let creation_time_av = AttributeValue::S(iso8601(user.get_creation_time()));
        let email_av = AttributeValue::S(user.get_email().clone());
        let wallet_address_av = AttributeValue::S(user.get_wallet_address().clone());

        let request = self
            .client
            .put_item()
            .table_name("users")
            .item("userID", user_id_av)
            .item("creationTime", creation_time_av)
            .item("walletAddress", wallet_address_av)
            .item("email_name", email_av)
            .item("device_name", device_av);

        //println!("Executing request [{request:?}] to add item...");

        match request.send().await {
            Ok(_) => {}
            Err(e) => {
                eprintln!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                panic!("error when saving at Dynamodb");
            }
        }

        Ok(())
    }

    async fn get_all(&self, page_number: u32, page_size: u32) -> Result<Vec<User>, Error> {
        let mut aux = Vec::new();
        aux.push(User::new());

        Ok(aux)
    }

    async fn get_by_user_id(&self, id: String) -> Result<Option<User>, Error> {
        let user_id_av = AttributeValue::S(id);
        let request = self
            .client
            .query()
            .table_name("users")
            .key_condition_expression("userID = :value".to_string())
            .expression_attribute_values(":value".to_string(), user_id_av)
            .select(Select::AllAttributes);

        match request.send().await {
            Ok(doc) => {
                if doc.count() > 0 {
                    println!("{:?}", doc.items.unwrap_or_default().pop());
                    let income = doc.items.unwrap_or_default().pop().unwrap();
                    
                    let mut user = User::new();
                    let aux = income.get("email").unwrap();
                    let i = aux.as_s().unwrap();
                    *user.set_email() = i.clone();
                    Ok(Some(user))
                } else {
                    Ok((None))
                }
            }
            Err(e) => {
                eprintln!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                panic!("error when saving at Dynamodb");
            }
        }
    }
}

fn iso8601(st: &std::time::SystemTime) -> String {
    let dt: DateTime<Utc> = st.clone().into();
    format!("{}", dt.format("%+"))
    // formats like "2001-07-08T00:34:60.026490+09:30"
}

/*
pub fn CreateUserRepo() -> UsersRepo {
    let aux = UsersRepo { client: todo!()  };

    aux.configure();

    return aux;

}
*/
