use std::{io::Error };

use crate::users::models::user::User;
use async_trait::async_trait;
use aws_config::{SdkConfig};
use aws_sdk_dynamodb::{
    model::{AttributeValue, Select},
    Client,
};
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
        let user_id_av = AttributeValue::S(user.user_id().clone());
        let device_av = AttributeValue::S(user.device().clone());
        let creation_time_av = AttributeValue::S(iso8601(user.creation_time()));
        let email_av = AttributeValue::S(user.email().clone());
        let wallet_address_av = AttributeValue::S(user.wallet_address().clone());

        let request = self
            .client
            .put_item()
            .table_name("users")
            .item("userID", user_id_av)
            .item("creationTime", creation_time_av)
            .item("walletAddress", wallet_address_av)
            .item("email", email_av)
            .item("device", device_av);

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

        let results = request.send().await;

        if let Some(items) = results.unwrap().items {
            let mut user = User::new();

            if items.len() == 0 {
                return Ok(None);
            } 

            let aux = &items[0];
            //let keyfield = "userID";
            //let eres = aux.get(&*keyfield).unwrap();
            let _user_id = aux.get("userID").unwrap();
            let user_id = _user_id.as_s().unwrap();

            let email = aux.get("email").unwrap().as_s().unwrap();
            let device = aux.get("device").unwrap().as_s().unwrap();
            let wallet_address = aux.get("walletAddress").unwrap().as_s().unwrap();
            let creation_time = aux.get("creationTime").unwrap().as_s().unwrap();

            user.set_creation_time(&from_iso8601(creation_time));
            user.set_device(device);
            user.set_wallet_address(wallet_address);
            user.set_email(email);
            user.set_user_id(user_id);
/* 
            let user = User {
                user_id: userID.clone(),
                email: email.clone(),
                device: device.clone(),
                wallet_address: wallet_address.clone(),
                creation_time: from_iso8601( creation_time)

            };*/
            Ok(Some(user))
        /*
                    for &item in &items[0] {
                        match item.get() {
                            Some(doc) => {
                                let i = doc.as_s().unwrap().as_str();
                                Ok(None)
                            }
                            None => Ok(None),
                        }
                    }
        */
        //let movies = items.iter().map(|v| v.into()).cloned().collect::<Option<User>>();
        //Ok(movies)
        //Ok(None)
        } else {
            Ok(None)
        }

        /*
        match request.send().await {
            Ok(doc) => {
                if doc.count() > 0 {
                    println!("{:?}", doc.items.unwrap_or_default().pop());
                    let income = doc.items.unwrap_or_default().pop(); //.clone();

                    match income {
                        Some(x) => {
                            let mut user = User::new();
                            let aux = x.get("email").unwrap();
                            let i = aux.as_s().unwrap();
                            *user.set_email() = i.clone();
                            Ok(Some(user))
                        }
                        None => Ok((None)),
                    }
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
        }*/
    }
}

//fn iso8601(st: &std::time::SystemTime) -> String {
fn iso8601(st: &DateTime<Utc>) -> String {
    let dt: DateTime<Utc> = st.clone().into();
    format!("{}", dt.format("%+"))
    // formats like "2001-07-08T00:34:60.026490+09:30"
}

fn from_iso8601(st: &String) -> DateTime<Utc> {

    let aux =  st.parse::<DateTime<Utc>>().unwrap();    //DateTime::parse_from_str(st, "%+").unwrap();
    aux
}

/*
pub fn CreateUserRepo() -> UsersRepo {
    let aux = UsersRepo { client: todo!()  };

    aux.configure();

    return aux;

}
*/
