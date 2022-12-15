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

static USERS_TABLE_NAME: &str = "usres";
static EMAIL_FIELD_NAME: &str = "email";
static DEVICE_FIELD_NAME: &str = "device";
static WALLETADDRESS_FIELD_NAME: &str = "walletAddress";
static USERID_FIELD_NAME: &str = "userID";
static CREATIONTIME_FIELD_NAME: &str = "creationTime";

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[async_trait]
pub trait UserRepository {
    async fn add_user(&self, user: &mut User) -> ResultE<String>;
    async fn get_by_user_id(&self, id: String) -> ResultE<Option<User>>;
    async fn get_all(&self, page_number: u32, page_size: u32) -> ResultE<Vec<User>>;
    async fn check_if_key_exists(&self, field: &str, value: &String) -> ResultE<Option<bool>>;
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
                //return Err(e.into());
                //Err(UserAlreadyExistsError.into())
                //UserAlreadyExistsError.into()
                //e.into()
                //DynamoDBError::into("eeerer")
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

        let request = self
            .client
            .put_item()
            .table_name(USERS_TABLE_NAME)
            .item(USERID_FIELD_NAME, user_id_av)
            .item(CREATIONTIME_FIELD_NAME, creation_time_av)
            .item(WALLETADDRESS_FIELD_NAME, wallet_address_av)
            .item(EMAIL_FIELD_NAME, email_av)
            .item(DEVICE_FIELD_NAME, device_av);

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
        let mut aux = Vec::new();
        aux.push(User::new());

        Ok(aux)
    }

    async fn get_by_user_id(&self, id: String) -> ResultE<Option<User>> {
        let user_id_av = AttributeValue::S(id);

        let mut filter = "".to_string();
        filter.push_str(USERID_FIELD_NAME);
        filter.push_str(" = :value");

        let request = self
            .client
            .query()
            .table_name(USERS_TABLE_NAME)
            .key_condition_expression(filter)
            .expression_attribute_values(":value".to_string(), user_id_av)
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
                return Err(DynamoDBError(e.to_string()).into())
            },
            Ok(_) => {}
        }

        if let Some(items) = results.unwrap().items {
            let mut user = User::new();

            if items.len() == 0 {
                return Ok(None);
            }

            let aux = &items[0];
            //let keyfield = "userID";
            //let eres = aux.get(&*keyfield).unwrap();
            let _user_id = aux.get(USERID_FIELD_NAME).unwrap();
            let user_id = _user_id.as_s().unwrap();

            let email = aux.get(EMAIL_FIELD_NAME).unwrap().as_s().unwrap();
            let device = aux.get(DEVICE_FIELD_NAME).unwrap().as_s().unwrap();
            let wallet_address = aux.get(WALLETADDRESS_FIELD_NAME).unwrap().as_s().unwrap();
            let creation_time = aux.get(CREATIONTIME_FIELD_NAME).unwrap().as_s().unwrap();

            user.set_creation_time(&from_iso8601(creation_time));
            user.set_device(device);
            user.set_wallet_address(wallet_address);
            user.set_email(email);
            user.set_user_id(user_id);

            Ok(Some(user))
        } else {
            Ok(None)
        }
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
