use std::collections::HashMap;

use crate::users::errors::users::{DynamoDBError, UserAlreadyExistsError};
use crate::users::models::user::User;
use async_trait::async_trait;
use aws_config::SdkConfig;
use aws_sdk_dynamodb::{
    model::{AttributeValue, Select},
    Client,
};
use chrono::{
    prelude::{DateTime, Utc},
    Local,
};

static USERS_TABLE_NAME: &str = "users";
static EMAIL_FIELD_NAME: &str = "email";
static DEVICE_FIELD_NAME: &str = "device";
static WALLETADDRESS_FIELD_NAME: &str = "walletAddress";
static USERID_FIELD_NAME: &str = "userID";
static CREATIONTIME_FIELD_NAME: &str = "creationTime";
static ROLES_FIELD_NAME: &str = "userRoles";

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[async_trait]
pub trait UserRepository {
    async fn add_user(&self, user: &mut User) -> ResultE<String>;
    async fn get_by_user_id(&self, id: String) -> ResultE<Option<User>>;
    async fn get_all(&self, page_number: u32, page_size: u32) -> ResultE<Vec<User>>;
    async fn check_if_key_exists(&self, field: &str, value: &String) -> ResultE<Option<bool>>;
    async fn get_by_filter(&self, field: &String, value: &String) -> ResultE<Vec<User>>;
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
        return aux;
    }
}

#[async_trait]
impl UserRepository for UsersRepo {
    async fn check_if_key_exists(&self, field: &str, value: &String) -> ResultE<Option<bool>> {
        let mut filter = "".to_string();
        filter.push_str(&field);
        filter.push_str(" = :value");

        let value_av = AttributeValue::S(value.clone());
        let request = self
            .client
            .query()
            .table_name(USERS_TABLE_NAME)
            .key_condition_expression(filter)
            .expression_attribute_values(":value".to_string(), value_av)
            .select(Select::Count);

        match request.send().await {
            Ok(_) => {
                return Ok(Some(true));
            }
            Err(e) => {
                tracing::error!("Failed to execute query for searching duplicates: {:?}", e);
                return Err(DynamoDBError(e.to_string()).into());
            }
        }
    }

    async fn add_user(&self, user: &mut User) -> ResultE<String> {
        let mut res = self
            .check_if_key_exists(EMAIL_FIELD_NAME, user.email())
            .await?;
        match res {
            Some(val) => {
                if val {
                    return Err(UserAlreadyExistsError("email already exists".to_string()).into());
                }
            }
            None => {}
        }
        res = self
            .check_if_key_exists(DEVICE_FIELD_NAME, user.device())
            .await?;
        match res {
            Some(val) => {
                if val {
                    return Err(UserAlreadyExistsError("device already exists".to_string()).into());
                }
            }
            None => {}
        }
        res = self
            .check_if_key_exists(WALLETADDRESS_FIELD_NAME, user.wallet_address())
            .await?;
        match res {
            Some(val) => {
                if val {
                    return Err(UserAlreadyExistsError(
                        "wallet address already exists".to_string(),
                    )
                    .into());
                }
            }
            None => {}
        }

        let user_id_av = AttributeValue::S(user.user_id().clone());
        let device_av = AttributeValue::S(user.device().clone());
        let creation_time_av = AttributeValue::S(iso8601(user.creation_time()));
        let email_av = AttributeValue::S(user.email().clone());
        let wallet_address_av = AttributeValue::S(user.wallet_address().clone());
        let roles_av = AttributeValue::S(user.roles().to_string());

        let request = self
            .client
            .put_item()
            .table_name(USERS_TABLE_NAME)
            .item(USERID_FIELD_NAME, user_id_av)
            .item(CREATIONTIME_FIELD_NAME, creation_time_av)
            .item(WALLETADDRESS_FIELD_NAME, wallet_address_av)
            .item(EMAIL_FIELD_NAME, email_av)
            .item(DEVICE_FIELD_NAME, device_av)
            .item(ROLES_FIELD_NAME, roles_av);

        //println!("Executing request [{request:?}] to add item...");

        match request.send().await {
            Ok(_) => Ok(user.user_id().clone()),
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(DynamoDBError(e.to_string()).into());
            }
        }
    }

    async fn get_all(&self, page_number: u32, page_size: u32) -> ResultE<Vec<User>> {
        let mut usersqueried = Vec::new();
        // result.push(User::new());

        /*let mut filter = "".to_string();
        filter.push_str(USERID_FIELD_NAME);
        filter.push_str(" = :hashKey");*/
        let mut page = 0;
        let mut exclusive_start_key = None;
        loop {
            let request = self
                .client
                .query()
                .table_name(USERS_TABLE_NAME)
                .limit(page_size as i32)
                .set_exclusive_start_key(exclusive_start_key)
                //.key_condition_expression(filter)
                //.expression_attribute_values(":value".to_string(), user_id_av)
                .select(Select::AllAttributes);

            let results = request.send().await;
            match results {
                Err(e) => {
                    let mssag = format!(
                        "Error at [{}] - {} ",
                        Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                        e
                    );
                    tracing::error!(mssag);
                    return Err(DynamoDBError(e.to_string()).into());
                }
                Ok(result) => {
                    if let Some(docs) = result.items {
                        if page == page_number {
                            for doc in docs {
                                let mut user = User::new();

                                mapping_from_doc_to_user(&doc, &mut user);

                                usersqueried.push(user);
                            }

                            break;
                        }
                        match result.last_evaluated_key {
                            Some(last_evaluated_key) => {
                                exclusive_start_key = Some(last_evaluated_key.clone());
                            }
                            None => {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
            page += 1;
        }

        Ok(usersqueried)
    }

    async fn get_by_user_id(&self, id: String) -> ResultE<Option<User>> {
        let user_id_av = AttributeValue::S(id);

        let request = self
            .client
            .get_item()
            .table_name(USERS_TABLE_NAME)
            .key(USERID_FIELD_NAME, user_id_av);

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(DynamoDBError(e.to_string()).into());
            }
            Ok(_) => {}
        }

        if let Some(aux) = results.unwrap().item {
            let mut user = User::new();
            
            mapping_from_doc_to_user(&aux, &mut user);

            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    async fn get_by_filter(&self, field: &String, value: &String) -> ResultE<Vec<User>> {
        let mut usersqueried = Vec::new();
        let value_av = AttributeValue::S(value.to_string());

        let mut filter = "".to_string();
        filter.push_str(&*field);
        filter.push_str(" = :value");

        let request = self
            .client
            .query()
            .table_name(USERS_TABLE_NAME)
            .key_condition_expression(filter)
            .expression_attribute_values(":value".to_string(), value_av)
            .select(Select::AllAttributes);

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(DynamoDBError(e.to_string()).into());
            }
            Ok(items) => {
                let docus = items.items().unwrap();
                for doc in docus {
                    let mut user = User::new();
                    
                    mapping_from_doc_to_user(&doc, &mut user);
                   
                    usersqueried.push(user);
                }
            }
        }
        Ok(usersqueried)
    }
}

//fn iso8601(st: &std::time::SystemTime) -> String {
fn iso8601(st: &DateTime<Utc>) -> String {
    let dt: DateTime<Utc> = st.clone().into();
    format!("{}", dt.format("%+"))
    // formats like "2001-07-08T00:34:60.026490+09:30"
}

fn from_iso8601(st: &String) -> DateTime<Utc> {
    let aux = st.parse::<DateTime<Utc>>().unwrap(); //DateTime::parse_from_str(st, "%+").unwrap();
    aux
}
fn mapping_from_doc_to_user(doc: &HashMap<String, AttributeValue>, user: &mut User) {
    let _user_id = doc.get(USERID_FIELD_NAME).unwrap();
    let user_id = _user_id.as_s().unwrap();

    let email = doc.get(EMAIL_FIELD_NAME).unwrap().as_s().unwrap();
    let device = doc.get(DEVICE_FIELD_NAME).unwrap().as_s().unwrap();
    let wallet_address = doc.get(WALLETADDRESS_FIELD_NAME).unwrap().as_s().unwrap();
    let creation_time = doc.get(CREATIONTIME_FIELD_NAME).unwrap().as_s().unwrap();

    user.set_creation_time(&from_iso8601(creation_time));
    user.set_device(device);
    user.set_wallet_address(wallet_address);
    user.set_email(email);
    user.set_user_id(user_id);
}
