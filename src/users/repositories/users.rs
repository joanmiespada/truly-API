
use std::collections::HashMap;

use argon2::{self };

use uuid::Uuid;

use crate::config::{Config, EnvironmentVariables};
use crate::users::errors::users::{
    DynamoDBError, UserAlreadyExistsError,  UserNoExistsError, UserParamNotAccepted, 
};
use crate::users::models::user::{User, UserRoles, UserStatus};
use async_trait::async_trait;
use aws_sdk_dynamodb::{
    model::{AttributeValue, Select},
    Client,
};
use chrono::{
    prelude::{DateTime, Utc},
    Local,
};


static USERS_TABLE_NAME: &str = "users";
static USERS_TABLE_INDEX_EMAIL: &str = "email";
static USERS_TABLE_INDEX_DEVICE: &str = "device";
static USERS_TABLE_INDEX_WALLETADDRESS: &str = "walletAddress";
static EMAIL_FIELD_NAME: &str = "email";
static PASSWORD_FIELD_NAME: &str = "password";
//static PASSWORD_SALT_FIELD_NAME: &str = "salt";
static DEVICE_FIELD_NAME: &str = "device";
static WALLETADDRESS_FIELD_NAME: &str = "walletAddress";
static USERID_FIELD_NAME_PK: &str = "userID";
static CREATIONTIME_FIELD_NAME: &str = "creationTime";
static LASTUPDATETIME_FIELD_NAME: &str = "lastUpdateTime";
static ROLES_FIELD_NAME: &str = "userRoles";
static STATUS_FIELD_NAME: &str = "userStatus";

static NULLABLE: &str = "__NULL__";

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[async_trait]
pub trait UserRepository {
    async fn add_user(&self, user: &mut User, password: &Option<String>) -> ResultE<String>;
    async fn update_user(&self, id: &String, user: &User) -> ResultE<bool>;
    async fn update_password(&self, id: &String, password: &String) -> ResultE<()>;
    async fn get_by_user_id(&self, id: &String) -> ResultE<User>;
    async fn get_by_user_device(&self, device: &String) -> ResultE<User>;
    async fn get_by_user_email_and_password(
        &self,
        email: &String,
        password: &String,
    ) -> ResultE<User>;
    async fn get_all(&self, page_number: u32, page_size: u32) -> ResultE<Vec<User>>;
    /*async fn check_if_key_exists(
        &self,
        field: &str,
        value: &String,
    ) -> ResultE<Option<Vec<String>>>;*/
    //async fn get_by_filter(&self, field: &str, value: &String, index: &str) -> ResultE<Vec<String>>;
}

#[derive(Clone, Debug)]
pub struct UsersRepo {
    client: Client,
    //config: Config
    environment_vars: EnvironmentVariables,
}

impl UsersRepo {
    /*
    pub fn new(aux: &SdkConfig) -> UsersRepo {
        UsersRepo {
            client: Client::new(aux),
        }
    }*/
    pub fn new(conf: &Config) -> UsersRepo {
        UsersRepo {
            client: Client::new(conf.aws_config()),
            environment_vars: conf.env_vars().clone(),
        }
    }
    async fn get_by_filter(
        &self,
        field: &str,
        value: &String,
        index: &str,
    ) -> ResultE<Vec<String>> {
        let mut usersqueried = Vec::new();
        let value_av = AttributeValue::S(value.to_string());

        let mut filter = "".to_string();
        filter.push_str(&*field);
        filter.push_str(" = :value");

        let request = self
            .client
            .query()
            .table_name(USERS_TABLE_NAME)
            .index_name(index)
            .key_condition_expression(filter)
            .expression_attribute_values(":value".to_string(), value_av)
            .select(Select::AllProjectedAttributes);

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
                    let _user_id = doc.get(USERID_FIELD_NAME_PK).unwrap();
                    let user_id = _user_id.as_s().unwrap();

                    usersqueried.push(user_id.clone());
                }
            }
        }
        Ok(usersqueried)
    }

    async fn check_duplicates(&self, user: &User) -> ResultE<bool> {
        match user.email() {
            None => {}
            Some(email) => {
                let res = self
                    .get_by_filter(EMAIL_FIELD_NAME, email, USERS_TABLE_INDEX_EMAIL)
                    .await?;
                if res.into_iter().filter(|x| *x != *user.user_id()).count() > 0 {
                    return Err(
                        UserAlreadyExistsError("email is already in use".to_string()).into(),
                    );
                }
            }
        }

        match user.device() {
            None => {}
            Some(device) => {
                let res = self
                    .get_by_filter(DEVICE_FIELD_NAME, device, USERS_TABLE_INDEX_DEVICE)
                    .await?;
                if res.into_iter().filter(|x| *x != *user.user_id()).count() > 0 {
                    return Err(UserAlreadyExistsError("device already exists".to_string()).into());
                }
            }
        }

        match user.wallet_address() {
            None => {}
            Some(wallet_address) => {
                let res = self
                    .get_by_filter(
                        WALLETADDRESS_FIELD_NAME,
                        wallet_address,
                        USERS_TABLE_INDEX_WALLETADDRESS,
                    )
                    .await?;
                if res.into_iter().filter(|x| *x != *user.user_id()).count() > 0 {
                    return Err(UserAlreadyExistsError(
                        "wallet address already exists".to_string(),
                    )
                    .into());
                }
            }
        }

        return Ok(true);
    }

    async fn get_by_id(&self, id: &String) -> ResultE<HashMap<String, AttributeValue>> {
        let user_id_av = AttributeValue::S(id.clone());

        let request = self
            .client
            .get_item()
            .table_name(USERS_TABLE_NAME)
            .key(USERID_FIELD_NAME_PK, user_id_av);

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
        match results.unwrap().item {
            None => Err(UserNoExistsError("id doesn't exist".to_string()).into()),
            Some(aux) => {
                //let mut user = User::new();
                //mapping_from_doc_to_user(&aux, &mut user);
                Ok(aux)
            }
        }
    }
}

#[async_trait]
impl UserRepository for UsersRepo {
    async fn add_user(&self, user: &mut User, password: &Option<String>) -> ResultE<String> {
        /*
        match user.email() {
            None => {}
            Some(email) => {
                let res = self
                    .get_by_filter(EMAIL_FIELD_NAME, email, USERS_TABLE_INDEX_EMAIL)
                    .await?;

                if res.len() > 0 {
                    return Err(UserAlreadyExistsError("email already exists".to_string()).into());
                }
            }
        }

        match user.device() {
            None => {}
            Some(device) => {
                let res = self
                    .get_by_filter(DEVICE_FIELD_NAME, device, USERS_TABLE_INDEX_DEVICE)
                    .await?;
                if res.len() > 0 {
                    return Err(UserAlreadyExistsError("device already exists".to_string()).into());
                }
            }
        }

        match user.wallet_address() {
            None => {}
            Some(wallet_address) => {
                let res = self
                    .get_by_filter(
                        WALLETADDRESS_FIELD_NAME,
                        wallet_address,
                        USERS_TABLE_INDEX_WALLETADDRESS,
                    )
                    .await?;
                if res.len() > 0 {
                    return Err(UserAlreadyExistsError(
                        "wallet address already exists".to_string(),
                    )
                    .into());
                }
            }
        }*/

        self.check_duplicates(user).await?;

        let user_id_av = AttributeValue::S(user.user_id().clone());
        let device_av = AttributeValue::S(user.device().clone().unwrap_or_default());
        let creation_time_av = AttributeValue::S(iso8601(user.creation_time()));
        let update_time_av = AttributeValue::S(iso8601(user.creation_time()));
        let email_av = AttributeValue::S(user.email().clone().unwrap_or_else( || NULLABLE.to_string()));
        let password_av: AttributeValue;
        //let password_salt_av: AttributeValue;

        match password {
            None => {
                password_av = AttributeValue::S( NULLABLE.to_string() ); //  "NULL".to_string());
                //password_salt_av =  AttributeValue::S("NULL".to_string());
            }
            Some(pass) => {
                let hash = cypher_text(pass, &self.environment_vars.hmac_secret)?;
                password_av =  AttributeValue::S(hash);
                //password_salt_av =  AttributeValue::S(salt);
            }
        }

        let wallet_address_av =
            AttributeValue::S(user.wallet_address().clone().unwrap_or_else( || NULLABLE.to_string()));
        let roles_av = AttributeValue::Ss(UserRoles::to_vec_str(user.roles()).clone());

        let status_av = AttributeValue::S(  user.status().to_string() );

        let request = self
            .client
            .put_item()
            .table_name(USERS_TABLE_NAME)
            .item(USERID_FIELD_NAME_PK, user_id_av)
            .item(CREATIONTIME_FIELD_NAME, creation_time_av)
            .item(LASTUPDATETIME_FIELD_NAME, update_time_av)
            .item(WALLETADDRESS_FIELD_NAME, wallet_address_av)
            .item(EMAIL_FIELD_NAME, email_av)
            .item(PASSWORD_FIELD_NAME, password_av)
            //.item(PASSWORD_SALT_FIELD_NAME, password_salt_av)
            .item(DEVICE_FIELD_NAME, device_av)
            .item(ROLES_FIELD_NAME, roles_av)
            .item(STATUS_FIELD_NAME, status_av);

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

        /*let user_id_av = AttributeValue::S(user.user_id().clone());
        let mut filter = "".to_string();
        filter.push_str(USERID_FIELD_NAME_PK);
        filter.push_str(" = :value");*/

        //let mut page = 0;
        //let mut exclusive_start_key = None;
        //loop {
            let results = self
                .client
                //.query()
                .scan()
                .table_name(USERS_TABLE_NAME)
                //.into_paginator()
                //.items()
                .send()
                ////.collect()
                .await;
                //.limit(page_size as i32)
                //.set_exclusive_start_key(exclusive_start_key)
                //.key_condition_expression(filter)
                //.expression_attribute_values(":value".to_string(), user_id_av)
                //.select(Select::AllAttributes);
            //.items();
            //.send()
            //.collect()
            //.await;

            //let results = request.send().await;
            //let results: Result<Vec<_>, _> = request.send().collect().await;
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
                       // if page == page_number {
                            for doc in docs {
                                let mut user = User::new();

                                mapping_from_doc_to_user(&doc, &mut user);

                                usersqueried.push(user.clone());
                            }

                            //break;
                        //}
                        /* 
                        match result.last_evaluated_key {
                            Some(last_evaluated_key) => {
                                exclusive_start_key = Some(last_evaluated_key.clone());
                            }
                            None => { _ }
                               // break;
                            
                        }*/
                    } else {
                       // break;
                    }
                }
            }
            //page += 1;
        //}

        Ok(usersqueried)
    }

    async fn get_by_user_id(&self, id: &String) -> ResultE<User> {
        let res = self.get_by_id(id).await?;
        let mut user = User::new();
        mapping_from_doc_to_user(&res, &mut user);
        Ok(user)
    }

    /*
        let user_id_av = AttributeValue::S(id.clone());

        let request = self
            .client
            .get_item()
            .table_name(USERS_TABLE_NAME)
            .key(USERID_FIELD_NAME_PK, user_id_av);

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
        match results.unwrap().item {
            None => Err(UserNoExistsError("id doesn't exist".to_string()).into()),
            Some(aux) => {
                let mut user = User::new();
                mapping_from_doc_to_user(&aux, &mut user);
                Ok(user)
            }
        }
    }
    */

    async fn get_by_user_device(&self, device: &String) -> ResultE<User> {
        let res = self
            .get_by_filter(&DEVICE_FIELD_NAME, device, &USERS_TABLE_INDEX_DEVICE)
            .await?;

        if res.len() == 0 {
            Err(UserNoExistsError("no device found".to_string()).into())
        } else {
            let user = self.get_by_user_id(&res[0]).await?;
            Ok(user)
        }
    }

    #[tracing::instrument(
        skip(password),
        fields(email=email)
    )]
    async fn get_by_user_email_and_password(
        &self,
        email: &String,
        password: &String,
    ) -> ResultE<User> {

        if email == NULLABLE {
            return Err(UserParamNotAccepted("email".to_string()).into());
        }
        if password == NULLABLE{
            return Err(UserParamNotAccepted("password".to_string()).into());
        }


        let email_av = AttributeValue::S(email.to_string());

        let mut filter = "".to_string();
        filter.push_str(&EMAIL_FIELD_NAME.to_string());
        filter.push_str(" = :value");

        let request = self
            .client
            .query()
            .table_name(USERS_TABLE_NAME)
            .index_name(USERS_TABLE_INDEX_EMAIL)
            .key_condition_expression(filter)
            .expression_attribute_values(":value".to_string(), email_av)
            .select(Select::AllProjectedAttributes);

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
                if docus.len() == 0 {
                    return Err(UserNoExistsError("no email found".to_string()).into());
                }
                let doc = docus.first().unwrap();
                let _user_id = doc.get(USERID_FIELD_NAME_PK).unwrap();
                let user_id = _user_id.as_s().unwrap();

                let user = self.get_by_id(&user_id).await?;

                let password_stored_hashed = user.get(PASSWORD_FIELD_NAME).unwrap().as_s().unwrap();
                //let password_salt = user.get(PASSWORD_SALT_FIELD_NAME).unwrap().as_s().unwrap();
                let password_ok = cypher_check(&password, &password_stored_hashed, &self.environment_vars.hmac_secret)?;
                if password_ok {
                    let mut usr = User::new();
                    mapping_from_doc_to_user(&user, &mut usr);
                    Ok(usr)
                } else {
                    Err(UserNoExistsError("no email or password found".to_string()).into())
                }
            }
        }
    }

    async fn update_user(&self, id: &String, user: &User) -> ResultE<bool> {
        self.check_duplicates(user).await?;
        /*
        let mut res = self
            .get_by_filter(EMAIL_FIELD_NAME, user.email(), USERS_TABLE_INDEX_EMAIL)
            //.check_if_key_exists(EMAIL_FIELD_NAME, user.email())
            .await?;
        if res.iter().filter(|x| **x == *id).count() != 0 {
            return Err(UserMismatchError("email is already in use".to_string()).into());
        }
        res = self
            .get_by_filter(DEVICE_FIELD_NAME, user.device(), USERS_TABLE_INDEX_DEVICE)
            //.check_if_key_exists(DEVICE_FIELD_NAME, user.device())
            .await?;
        if res.iter().filter(|x| **x == *id).count() != 0 {
            return Err(UserMismatchError("device is already in use".to_string()).into());
        }
        res = self
            .get_by_filter(
                WALLETADDRESS_FIELD_NAME,
                user.wallet_address(),
                USERS_TABLE_INDEX_WALLETADDRESS,
            )
            //.check_if_key_exists(WALLETADDRESS_FIELD_NAME, user.wallet_address())
            .await?;
        if res.iter().filter(|x| **x == *id).count() != 0 {
            return Err(
                UserMismatchError("wallet address is already already in use".to_string()).into(),
            );
        }*/

        let last_update_time_av = AttributeValue::S(iso8601(&Utc::now()));
        let id_av = AttributeValue::S(id.clone());

        let dvc = user.device().clone().unwrap_or_default();
        let device_av = AttributeValue::S(dvc);

        let email_av: AttributeValue = AttributeValue::S(user.email().clone().unwrap_or_default());

        let wallet_address_av: AttributeValue =
            AttributeValue::S(user.wallet_address().clone().unwrap_or_default());
        let roles_av = AttributeValue::Ss(UserRoles::to_vec_str(user.roles()).clone());

        let status_av: AttributeValue = AttributeValue::S( user.status().to_string());

        //format!("set {0} = :device ",DEVICE_FIELD_NAME, )
        let mut update_express = "set ".to_string();
        update_express.push_str(format!("{0} = :device, ", DEVICE_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :email, ", EMAIL_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :wa, ", WALLETADDRESS_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :lastup, ", LASTUPDATETIME_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :r_les, ", ROLES_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :_status ", STATUS_FIELD_NAME).as_str());


        let request = self
            .client
            .update_item()
            .table_name(USERS_TABLE_NAME)
            .key(USERID_FIELD_NAME_PK, id_av)
            .update_expression(update_express)
            .expression_attribute_values(":device", device_av)
            .expression_attribute_values(":email", email_av)
            .expression_attribute_values(":wa", wallet_address_av)
            .expression_attribute_values(":lastup", last_update_time_av)
            .expression_attribute_values(":r_les", roles_av)
            .expression_attribute_values(":_status", status_av);

        match request.send().await {
            Ok(_) => Ok(true),
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

    async fn update_password(&self, id: &String, password: &String) -> ResultE<()> {
        let aux = &self.environment_vars.hmac_secret;
        let hash = cypher_text(password, aux)?;
        let password_av = AttributeValue::S(hash);
        //let password_salt_av = AttributeValue::S(salt);

        let last_update_time_av = AttributeValue::S(iso8601(&Utc::now()));
        let id_av = AttributeValue::S(id.clone());

        let mut update_express = "set ".to_string();
        update_express.push_str(format!("{0} = :value1, ", PASSWORD_FIELD_NAME).as_str());
        //update_express.push_str(format!("{0} = :value2, ", PASSWORD_SALT_FIELD_NAME).as_str());
        update_express.push_str(format!("{0} = :lastup ", LASTUPDATETIME_FIELD_NAME).as_str());

        let request = self
            .client
            .update_item()
            .table_name(USERS_TABLE_NAME)
            .key(USERID_FIELD_NAME_PK, id_av)
            .update_expression(update_express)
            .expression_attribute_values(":value1", password_av)
            //.expression_attribute_values(":value2", password_salt_av)
            .expression_attribute_values(":lastup", last_update_time_av);

        match request.send().await {
            Ok(_) => Ok(()),
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
    let _user_id = doc.get(USERID_FIELD_NAME_PK).unwrap();
    let user_id = _user_id.as_s().unwrap();
    user.set_user_id(user_id);

    let email_t = doc.get(EMAIL_FIELD_NAME);//.unwrap().as_s().unwrap();
    match email_t{
        None => {},
        Some(email) =>{ user.set_email(email.as_s().unwrap());}
    }
    //let passw = doc.get(PASSWORD_FIELD_NAME).unwrap().as_s().unwrap();
    let device_t = doc.get(DEVICE_FIELD_NAME); //. unwrap_or_else( || &AttributeValue::S("".to_string())) .as_s().unwrap();
    match device_t{
        None => {},
        Some(device) => { user.set_device(device.as_s().unwrap());}
    }
    let wallet_address_t = doc.get(WALLETADDRESS_FIELD_NAME); //. unwrap_or_else( || &AttributeValue::S("".to_string()))  .as_s().unwrap();
    match wallet_address_t {    
        None => {},
        Some(wallet_address) => {  user.set_wallet_address(wallet_address.as_s().unwrap() ); }
    }
    let roles_t = doc.get(ROLES_FIELD_NAME); //.unwrap_or_else( || &AttributeValue::S("".to_string())).as_ss().unwrap();
    match roles_t {
        None=>{},
        Some(roles) => { user.set_roles(&UserRoles::from_vec_str(roles.as_ss().unwrap() ))}
    }
    let creation_time_t = doc.get(CREATIONTIME_FIELD_NAME);//.unwrap().as_s().unwrap();
    match creation_time_t  {
        None=> {},
        Some(creation_time) => { user.set_creation_time(&from_iso8601(creation_time.as_s().unwrap() ));    }
    }
    let status_t = doc.get(STATUS_FIELD_NAME).unwrap().as_s().unwrap();
    let aux = UserStatus::parse(status_t);
    match aux {
        Some(ut) =>{ user.set_status( &ut); }
        None => {}
    }
    

    //user.set_creation_time(&from_iso8601(creation_time));
    //user.set_device(device);
    //user.set_wallet_address(wallet_address);
    //user.set_email(email);
    //user.set_password(passw);
    //user.set_user_id(user_id);
    //user.set_roles(&UserRoles::from_vec_str(roles));
}

fn cypher_text(text: &String, key: &String) -> ResultE<String> {


    let salt = Uuid::new_v4().to_string();

    let config = argon2::Config {
        variant: argon2::Variant::Argon2i,
        version: argon2::Version::Version13,
        mem_cost: 65536,
        time_cost: 10,
        lanes: 4,
        thread_mode: argon2::ThreadMode::Parallel,
        secret: key.as_bytes(),
        ad: &[],
        hash_length: 32
    };
    
    let hash = argon2::hash_encoded(text.as_bytes(), salt.as_bytes(), &config).unwrap();

    //Ok((hash,salt))
    Ok(hash)

/* 
    let salt = SaltString::generate(&mut rand::thread_rng());

    let config = argon2::Config {
        variant: argon2::Variant::Argon2i,
        version: Version::Version13,
        mem_cost: 65536,
        time_cost: 10,
        lanes: 4,
        thread_mode: ThreadMode::Parallel,
        secret: &[],
        ad: &[],
        hash_length: 32
    };


    let password_hash = Argon2::default()
            .hash_password(text.as_bytes(), &salt)
            .unwrap()
            .to_string();

*/

    //type HmacSha256 = Hmac<Sha256>;

    //let hmac_key = env_vars.hmac_secret.as_bytes();

    // let hmac_key = key.as_bytes();
    // let  signatured_key = sha3::Sha3_256::digest  hmac::Key::new( hmac::HMAC_SHA256,  hmac_key);
    // let signature = hmac::sign(&signatured_key, text.as_bytes());

    // let res =BASE32HEX.encode(signature.as_ref());

    

    //let mut hasher = HmacSha256::new_from_slice(hmac_key);
    //let mut hmac = HmacSha256::from(hmac_key);

    //let mut hmac = Hmac::new(Sha256::new(), hmac_key);

   // hasher.update(text.as_bytes());

    //let res = hasher.result().code().to_hex(); // .to_owned();
    //return Ok(res);
    /*
        let cypher_password_ops = String::from_utf8(res); //. unwrap();
        match cypher_password_ops  {
            Err(e) => { eprintln!("{}",e.to_string());  return Err(e.into());},
            Ok(cypher_password) => Ok(cypher_password)

        }
    */
    //return Ok(cypher_password);
    /*   match cypher_password_ops {
    Err(x) =>{

    },
    Ok(cypher_password) => { return cypher_password}*/

    //}
    //.unwrap(); // result.into_bytes();
}

fn cypher_check(text_to_check: &String, already_ciphered: &String, key: &String) -> ResultE<bool> {


    //let password = b"password";
    //let salt = b"othersalt";
    /* 
    let config = argon2::Config {
        variant: argon2::Variant::Argon2i,
        version: argon2::Version::Version13,
        mem_cost: 65536,
        time_cost: 10,
        lanes: 4,
        thread_mode: argon2::ThreadMode::Parallel,
        secret: key.as_bytes(),
        ad: &[],
        hash_length: 32
    };*/
    //let hash = argon2::hash_encoded(text_to_check.as_bytes(), salt.as_bytes(), &config).unwrap();
    let matches = argon2::verify_encoded_ext(&already_ciphered, text_to_check.as_bytes(), key.as_bytes(), b"" ).unwrap();

    match matches {
        true => Ok(true),
        _ => Ok(false)
    } 

}

/* 

    let expected_password_hash = PasswordHash::new(already_ciphered);
    match expected_password_hash {
        Err(e) => Err( UserPasswordError("sdf".to_string()).into()),
        Ok(pass_hash) => {

        let aux = Argon2::default()
        .verify_password(
             text_to_check.as_bytes(), 
             &pass_hash
        ); //.map_err(UserPasswordError("".to_string()).into() );

        match aux {
            Err(_)=> Err(UserPasswordError("".to_string()).into()),
            Ok(ok)=>Ok(true)
        }        
    }
    
}
*/

/* 
    let hmac_key = key.as_bytes();
    let  signatured_key = hmac::Key::new( hmac::HMAC_SHA256,  hmac_key);
    //let signature = hmac::sign(&signatured_key, text_to_check.as_bytes());
    let res = hmac::verify(&signatured_key, text_to_check.as_bytes(), already_ciphered.as_bytes() );
    match res {
        Ok(_)=> Ok(true),
        Err(_)=> Ok(false)
    }
    */


    //hmac.input(text_to_check.as_bytes());

    //let ok = hmac.result().code().to_hex();


//}