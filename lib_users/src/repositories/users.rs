use std::collections::HashMap;

use argon2::{self};
use aws_sdk_dynamodb::operation::transact_write_items::builders::TransactWriteItemsFluentBuilder;
use aws_sdk_dynamodb::types::{Put, TransactWriteItem};
use tracing::error;
use uuid::Uuid;

use crate::errors::users::{
    UserAlreadyExistsError, UserDynamoDBError, UserNoExistsError, UserParamNotAccepted,
};
use crate::models::user::{User, UserRoles, UserStatus};
use async_trait::async_trait;
use aws_sdk_dynamodb::{
    types::{AttributeValue, Select},
    Client,
};
use chrono::{
    prelude::{DateTime, Utc},
    Local,
};
use lib_config::{config::Config, environment::EnvironmentVariables};

use super::schema_user::{
    LOGIN_DEVICE_FIELD_NAME_PK, LOGIN_DEVICE_TABLE_NAME, LOGIN_DEVICE_USERID_INDEX,
    LOGIN_EMAIL_FIELD_NAME_PK, LOGIN_EMAIL_TABLE_NAME, LOGIN_EMAIL_USERID_INDEX,
    LOGIN_WALLET_FIELD_NAME_PK, LOGIN_WALLET_TABLE_NAME, LOGIN_WALLET_USERID_INDEX,
    USERID_FIELD_NAME, USERID_FIELD_NAME_PK, USERS_TABLE_NAME,
};

static PASSWORD_FIELD_NAME: &str = "password";
static CREATIONTIME_FIELD_NAME: &str = "creationTime";
static LASTUPDATETIME_FIELD_NAME: &str = "lastUpdateTime";
static ROLES_FIELD_NAME: &str = "userRoles";
static STATUS_FIELD_NAME: &str = "userStatus";

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[async_trait]
pub trait UserRepository {
    async fn add(&self, user: &mut User, password: &Option<String>) -> ResultE<()>;
    async fn update(&self, id: &String, user: &User) -> ResultE<()>;
    async fn update_password(&self, id: &String, password: &String) -> ResultE<()>;
    async fn get_by_id(&self, id: &String) -> ResultE<User>;
    async fn get_by_device(&self, device: &String) -> ResultE<User>;
    async fn get_by_wallet_address(&self, wallet: &String) -> ResultE<User>;
    async fn get_by_email_and_password(&self, email: &String, password: &String) -> ResultE<User>;
    async fn get_all(&self, page_number: u32, page_size: u32) -> ResultE<Vec<User>>;
    async fn remove(&self, user_id: &String) -> ResultE<()>;
}

#[derive(Clone, Debug)]
pub struct UsersRepo {
    client: Client,
    //config: Config
    environment_vars: EnvironmentVariables,
}

impl UsersRepo {
    pub fn new(conf: &Config) -> UsersRepo {
        UsersRepo {
            client: Client::new(conf.aws_config()),
            environment_vars: conf.env_vars().clone(),
        }
    }
    async fn get_by_filter_key(
        &self,
        search_field: &str,
        search_value: &String,
        return_field: &str,
        table_name: &str,
    ) -> ResultE<Option<String>> {
        //let mut usersqueried = Vec::new();
        let value_av = AttributeValue::S(search_value.to_string());

        //let mut filter = "".to_string();
        //filter.push_str(&*field);
        //filter.push_str(" = :value");

        let request = self
            .client
            //.query()
            .get_item()
            .table_name(table_name) //USERS_TABLE_NAME)
            //.index_name(index)
            .key(search_field, value_av);
        //.select(Select::AllProjectedAttributes);

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(UserDynamoDBError(e.to_string()).into());
            }
            Ok(items) => {
                let doc_op = items.item();
                match doc_op {
                    None => Ok(None),
                    Some(doc) => {
                        //for doc in docus {
                        //let _user_id = doc.get(USERID_FIELD_NAME_PK).unwrap();
                        let data = doc.get(return_field); //USERID_FIELD_NAME).unwrap();
                        match data {
                            None => Ok(None),
                            Some(value) => {
                                let val = value.as_s().unwrap();
                                Ok(Some(val.clone()))
                            }
                        }
                    }
                }
                //usersqueried.push(user_id.clone());
                //}
            }
        }
        //Ok(usersqueried)
    }

    async fn get_by_filter_user_id(
        &self,
        index: &str,
        search_value: &String,
        return_field: &str,
        table_name: &str,
    ) -> ResultE<Option<String>> {
        //let mut usersqueried = Vec::new();
        let value_av = AttributeValue::S(search_value.to_string());

        let mut filter = "".to_string();
        filter.push_str(USERID_FIELD_NAME);
        filter.push_str(" = :value");

        let request = self
            .client
            .query()
            //.get_item()
            .table_name(table_name) //USERS_TABLE_NAME)
            .index_name(index)
            .key_condition_expression(filter)
            .expression_attribute_values(":value".to_string(), value_av)
            .select(Select::AllProjectedAttributes);
        //.key(search_field, value_av);
        //.select(Select::AllProjectedAttributes);

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(UserDynamoDBError(e.to_string()).into());
            }
            Ok(items) => {
                let doc_op = items.items();
                match doc_op {
                    None => Ok(None),
                    Some(docs) => {
                        if docs.len() == 0 {
                            Ok(None)
                        } else {
                            //for doc in docus {
                            //let _user_id = doc.get(USERID_FIELD_NAME_PK).unwrap();
                            let doc = docs[0].clone();
                            let data = doc.get(return_field); //USERID_FIELD_NAME).unwrap();
                            match data {
                                None => Ok(None),
                                Some(value) => {
                                    let val = value.as_s().unwrap();
                                    Ok(Some(val.clone()))
                                }
                            }
                        }
                    }
                }
                //usersqueried.push(user_id.clone());
                //}
            }
        }
        //Ok(usersqueried)
    }

    async fn check_duplicates(&self, user: &User) -> ResultE<bool> {
        match user.email() {
            None => {}
            Some(email) => {
                let res = self
                    .get_by_filter_key(
                        LOGIN_EMAIL_FIELD_NAME_PK,
                        email,
                        USERID_FIELD_NAME,
                        LOGIN_EMAIL_TABLE_NAME,
                    )
                    .await?;
                /*
                if res.into_iter().filter(|x| *x != *user.user_id()).count() > 0 {
                    return Err(
                        UserAlreadyExistsError("email is already in use".to_string()).into(),
                    );
                }*/
                match res {
                    None => {}
                    Some(_) => {
                        return Err(
                            UserAlreadyExistsError("email is already in use".to_string()).into(),
                        );
                    }
                }
            }
        }

        match user.device() {
            None => {}
            Some(device) => {
                let res = self
                    .get_by_filter_key(
                        LOGIN_DEVICE_FIELD_NAME_PK,
                        device,
                        USERID_FIELD_NAME,
                        LOGIN_DEVICE_TABLE_NAME,
                    )
                    .await?; /*
                             if res.into_iter().filter(|x| *x != *user.user_id()).count() > 0 {
                                 return Err(UserAlreadyExistsError("device already exists".to_string()).into());
                             }*/
                match res {
                    None => {}
                    Some(_) => {
                        return Err(
                            UserAlreadyExistsError("device is already in use".to_string()).into(),
                        );
                    }
                }
            }
        }

        match user.wallet_address() {
            None => {}
            Some(wallet_address) => {
                let res = self
                    .get_by_filter_key(
                        LOGIN_WALLET_FIELD_NAME_PK,
                        wallet_address,
                        USERID_FIELD_NAME,
                        LOGIN_WALLET_TABLE_NAME,
                    )
                    .await?;
                match res {
                    None => {}
                    Some(_) => {
                        return Err(UserAlreadyExistsError(
                            "wallet address is already in use".to_string(),
                        )
                        .into());
                    }
                }
            }
        }

        return Ok(true);
    }

    async fn get_by_id_hashmap(&self, id: &String) -> ResultE<HashMap<String, AttributeValue>> {
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
                return Err(UserDynamoDBError(e.to_string()).into());
            }
            Ok(_) => {}
        }
        match results.unwrap().item {
            None => Err(UserNoExistsError("id doesn't exist".to_string()).into()),
            Some(aux) => Ok(aux),
        }
    }

    async fn mapping_from_doc_to_user_logins(&self, id: &String, user: &mut User) -> ResultE<()> {
        let email_op = self
            .get_by_filter_user_id(
                LOGIN_EMAIL_USERID_INDEX,
                id,
                LOGIN_EMAIL_FIELD_NAME_PK,
                LOGIN_EMAIL_TABLE_NAME,
            )
            .await?;
        match email_op {
            None => {}
            Some(email) => user.set_email(&email),
        }
        let device_op = self
            .get_by_filter_user_id(
                LOGIN_DEVICE_USERID_INDEX,
                id,
                LOGIN_DEVICE_FIELD_NAME_PK,
                LOGIN_DEVICE_TABLE_NAME,
            )
            .await?;
        match device_op {
            None => {}
            Some(device) => user.set_device(&device),
        }
        let wallet_op = self
            .get_by_filter_user_id(
                LOGIN_WALLET_USERID_INDEX,
                id,
                LOGIN_WALLET_FIELD_NAME_PK,
                LOGIN_WALLET_TABLE_NAME,
            )
            .await?;
        match wallet_op {
            None => {}
            Some(wallet) => user.set_wallet_address(&wallet),
        }
        Ok(())
    }

    async fn new_or_update_builder(
        &self,
        user: &User,
        password: &Option<String>,
    ) -> ResultE<TransactWriteItemsFluentBuilder> {
        // TransactWriteItems> {
        let mut request = self.client.transact_write_items();

        let creation_time_av = AttributeValue::S(iso8601(user.creation_time()));
        let last_update_time_av = AttributeValue::S(iso8601(&Utc::now()));
        let id_av = AttributeValue::S(user.user_id().clone());
        let roles_av = AttributeValue::Ss(UserRoles::to_vec_str(user.roles()).clone());
        let status_av: AttributeValue = AttributeValue::S(user.status().to_string());

        let mut user_fields = Put::builder();
        user_fields = user_fields
            .item(USERID_FIELD_NAME_PK, id_av.clone())
            .item(CREATIONTIME_FIELD_NAME, creation_time_av)
            .item(LASTUPDATETIME_FIELD_NAME, last_update_time_av)
            .item(ROLES_FIELD_NAME, roles_av)
            .item(STATUS_FIELD_NAME, status_av);

        request = request.transact_items(
            TransactWriteItem::builder()
                .put(user_fields.table_name(USERS_TABLE_NAME).build())
                .build(),
        );

        match user.device() {
            None => {}
            Some(dvc) => {
                let device_av = AttributeValue::S(dvc.to_owned());

                let mut device_fields = Put::builder();
                device_fields = device_fields
                    .item(USERID_FIELD_NAME, id_av.clone())
                    .item(LOGIN_DEVICE_FIELD_NAME_PK, device_av);

                request = request.transact_items(
                    TransactWriteItem::builder()
                        .put(device_fields.table_name(LOGIN_DEVICE_TABLE_NAME).build())
                        .build(),
                );
            }
        }
        match user.wallet_address() {
            None => {}
            Some(wallet) => {
                let wallet_av = AttributeValue::S(wallet.to_owned());

                let mut wallet_fields = Put::builder();
                wallet_fields = wallet_fields
                    .item(USERID_FIELD_NAME, id_av.clone())
                    .item(LOGIN_WALLET_FIELD_NAME_PK, wallet_av);

                request = request.transact_items(
                    TransactWriteItem::builder()
                        .put(wallet_fields.table_name(LOGIN_WALLET_TABLE_NAME).build())
                        .build(),
                );
            }
        }
        match user.email() {
            None => {}
            Some(email) => {
                let mut email_fields = Put::builder();

                match password {
                    Some(password) => {
                        let hash = cypher_text(password, &self.environment_vars.hmac_secret())?;
                        let password_av: AttributeValue = AttributeValue::S(hash);
                        email_fields = email_fields.item(PASSWORD_FIELD_NAME, password_av);
                    }
                    None => {
                        let pass = self
                            .get_by_filter_user_id(
                                LOGIN_EMAIL_USERID_INDEX,
                                user.user_id(),
                                PASSWORD_FIELD_NAME,
                                LOGIN_EMAIL_TABLE_NAME,
                            )
                            .await?;
                        match pass {
                            None => {}
                            Some(password_db) => {
                                //it's already cyphered
                                let password_av: AttributeValue = AttributeValue::S(password_db);
                                email_fields = email_fields.item(PASSWORD_FIELD_NAME, password_av);
                            }
                        }
                    }
                }

                let email_av: AttributeValue = AttributeValue::S(email.to_owned());

                email_fields = email_fields
                    .item(USERID_FIELD_NAME, id_av.clone())
                    .item(LOGIN_EMAIL_FIELD_NAME_PK, email_av);

                request = request.transact_items(
                    TransactWriteItem::builder()
                        .put(email_fields.table_name(LOGIN_EMAIL_TABLE_NAME).build())
                        .build(),
                );
            }
        }
        Ok(request)
    }
}

#[async_trait]
impl UserRepository for UsersRepo {
    async fn add(&self, user: &mut User, password: &Option<String>) -> ResultE<()> {
        self.check_duplicates(user).await?;

        let request = self.new_or_update_builder(user, password).await?;
        match request.send().await {
            Ok(_updated) => {
                let mssag = format!(
                    "Record created at [{}] - item id: {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    user.user_id().to_string()
                );
                tracing::debug!(mssag);

                Ok(())
            }
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e.to_string()
                );
                error!(mssag);
                error!("{}", e);
                return Err(UserDynamoDBError(e.to_string()).into());
            }
        }
    }

    async fn get_all(&self, _page_number: u32, _page_size: u32) -> ResultE<Vec<User>> {
        let mut usersqueried = Vec::new();

        let results = self.client.scan().table_name(USERS_TABLE_NAME).send().await;

        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(UserDynamoDBError(e.to_string()).into());
            }
            Ok(result) => {
                if let Some(docs) = result.items {
                    for doc in docs {
                        let mut user = User::new();

                        mapping_from_doc_to_user(&doc, &mut user);
                        self.mapping_from_doc_to_user_logins(user.clone().user_id(), &mut user)
                            .await?;

                        usersqueried.push(user.clone());
                    }
                }
            }
        }

        Ok(usersqueried)
    }

    async fn get_by_id(&self, id: &String) -> ResultE<User> {
        let res = self.get_by_id_hashmap(id).await?;
        let mut user = User::new();
        mapping_from_doc_to_user(&res, &mut user);
        self.mapping_from_doc_to_user_logins(id, &mut user).await?;

        Ok(user)
    }

    async fn get_by_device(&self, device: &String) -> ResultE<User> {
        let res = self
            .get_by_filter_key(
                LOGIN_DEVICE_FIELD_NAME_PK,
                device,
                USERID_FIELD_NAME,
                LOGIN_DEVICE_TABLE_NAME,
            )
            .await?;
        match res {
            None => {
                return Err(UserNoExistsError("no device found".to_string()).into());
            }
            Some(user_id) => {
                let res = self.get_by_id_hashmap(&user_id).await?;
                let mut user = User::new();
                mapping_from_doc_to_user(&res, &mut user);
                user.set_device(device);

                let wallet_op = self
                    .get_by_filter_user_id(
                        LOGIN_WALLET_USERID_INDEX,
                        &user_id,
                        LOGIN_WALLET_FIELD_NAME_PK,
                        LOGIN_WALLET_TABLE_NAME,
                    )
                    .await?;
                match wallet_op {
                    None => {}
                    Some(wallet) => user.set_wallet_address(&wallet),
                }
                let email_op = self
                    .get_by_filter_user_id(
                        LOGIN_EMAIL_USERID_INDEX,
                        &user_id,
                        LOGIN_EMAIL_FIELD_NAME_PK,
                        LOGIN_EMAIL_TABLE_NAME,
                    )
                    .await?;
                match email_op {
                    None => {}
                    Some(email) => user.set_email(&email),
                }

                Ok(user)
            }
        }
    }

    async fn get_by_wallet_address(&self, wallet: &String) -> ResultE<User> {
        let res = self
            .get_by_filter_key(
                LOGIN_WALLET_FIELD_NAME_PK,
                wallet,
                USERID_FIELD_NAME,
                LOGIN_WALLET_TABLE_NAME,
            )
            .await?;
        match res {
            None => {
                return Err(UserNoExistsError("no wallet found".to_string()).into());
            }
            Some(user_id) => {
                let res = self.get_by_id_hashmap(&user_id).await?;
                let mut user = User::new();
                mapping_from_doc_to_user(&res, &mut user);
                user.set_wallet_address(wallet);

                let email_op = self
                    .get_by_filter_user_id(
                        LOGIN_EMAIL_USERID_INDEX,
                        &user_id,
                        LOGIN_EMAIL_FIELD_NAME_PK,
                        LOGIN_EMAIL_TABLE_NAME,
                    )
                    .await?;
                match email_op {
                    None => {}
                    Some(email) => user.set_email(&email),
                }
                let device_op = self
                    .get_by_filter_user_id(
                        LOGIN_DEVICE_USERID_INDEX,
                        &user_id,
                        LOGIN_DEVICE_FIELD_NAME_PK,
                        LOGIN_DEVICE_TABLE_NAME,
                    )
                    .await?;
                match device_op {
                    None => {}
                    Some(device) => user.set_device(&device),
                }

                Ok(user)
            }
        }
    }

    #[tracing::instrument(
        skip(password),
        fields(email=email)
    )]
    async fn get_by_email_and_password(&self, email: &String, password: &String) -> ResultE<User> {
        if email.is_empty() {
            return Err(UserParamNotAccepted("email".to_string()).into());
        }
        if password.is_empty() {
            return Err(UserParamNotAccepted("password".to_string()).into());
        }

        let res = self
            .get_by_filter_key(
                LOGIN_EMAIL_FIELD_NAME_PK,
                email,
                PASSWORD_FIELD_NAME,
                LOGIN_EMAIL_TABLE_NAME,
            )
            .await?;
        match res {
            None => {
                return Err(UserNoExistsError("no device found".to_string()).into());
            }
            Some(password_stored_hashed) => {
                let password_ok = cypher_check(
                    &password,
                    &password_stored_hashed,
                    self.environment_vars.hmac_secret(),
                )?;
                if password_ok {
                    let user_op = self
                        .get_by_filter_key(
                            LOGIN_EMAIL_FIELD_NAME_PK,
                            email,
                            USERID_FIELD_NAME,
                            LOGIN_EMAIL_TABLE_NAME,
                        )
                        .await?;
                    let user_id = user_op.unwrap();
                    let res = self.get_by_id_hashmap(&user_id).await?;

                    let mut user = User::new();
                    mapping_from_doc_to_user(&res, &mut user);
                    user.set_email(email);

                    let device_op = self
                        .get_by_filter_user_id(
                            LOGIN_DEVICE_USERID_INDEX,
                            &user_id,
                            LOGIN_DEVICE_FIELD_NAME_PK,
                            LOGIN_DEVICE_TABLE_NAME,
                        )
                        .await?;
                    match device_op {
                        None => {}
                        Some(device) => user.set_device(&device),
                    }
                    let wallet_op = self
                        .get_by_filter_user_id(
                            LOGIN_WALLET_USERID_INDEX,
                            &user_id,
                            LOGIN_WALLET_FIELD_NAME_PK,
                            LOGIN_WALLET_TABLE_NAME,
                        )
                        .await?;
                    match wallet_op {
                        None => {}
                        Some(wallet) => user.set_wallet_address(&wallet),
                    }

                    Ok(user)
                } else {
                    Err(UserNoExistsError("no email or password found".to_string()).into())
                }
            }
        }
    }

    async fn update(&self, id: &String, user: &User) -> ResultE<()> {
        //self.check_duplicates(user).await?;

        let request = self.new_or_update_builder(user, &None).await?;

        match request.send().await {
            Ok(_updated) => {
                let mssag = format!(
                    "Record updated at [{}] - item id: {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    id.to_string()
                );
                tracing::debug!(mssag);

                Ok(())
            }
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e.to_string()
                );
                error!(mssag);
                error!("{}", e);
                return Err(UserDynamoDBError(e.to_string()).into());
            }
        }
    }

    async fn update_password(&self, id: &String, password: &String) -> ResultE<()> {
        let user = self.get_by_id(id).await?;
        let pass = Some(password.to_owned());
        let request = self.new_or_update_builder(&user, &pass).await?;

        match request.send().await {
            Ok(_updated) => {
                let mssag = format!(
                    "Record updated at [{}] - item id: {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    id.to_string()
                );
                tracing::debug!(mssag);

                Ok(())
            }
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e.to_string()
                );
                error!(mssag);
                error!("{}", e);
                return Err(UserDynamoDBError(e.to_string()).into());
            }
        }
    }

    async fn remove(&self, id: &String) -> ResultE<()> {
        let user_id_av = AttributeValue::S(id.clone());

        let request = self
            .client
            .delete_item()
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
                return Err(UserDynamoDBError(e.to_string()).into());
            }
            Ok(_) => {
                return Ok(());
            }
        }
    }
}

fn iso8601(st: &DateTime<Utc>) -> String {
    let dt: DateTime<Utc> = st.clone().into();
    format!("{}", dt.format("%+"))
}

fn from_iso8601(st: &String) -> DateTime<Utc> {
    let aux = st.parse::<DateTime<Utc>>().unwrap();
    aux
}
fn mapping_from_doc_to_user(doc: &HashMap<String, AttributeValue>, user: &mut User) {
    let _user_id = doc.get(USERID_FIELD_NAME_PK).unwrap();
    let user_id = _user_id.as_s().unwrap();
    user.set_user_id(user_id);

    let roles_t = doc.get(ROLES_FIELD_NAME);
    match roles_t {
        None => {}
        Some(roles) => user.set_roles(&UserRoles::from_vec_str(roles.as_ss().unwrap())),
    }
    let creation_time_t = doc.get(CREATIONTIME_FIELD_NAME);
    match creation_time_t {
        None => {}
        Some(creation_time) => {
            user.set_creation_time(&from_iso8601(creation_time.as_s().unwrap()));
        }
    }

    let last_update_time_t = doc.get(LASTUPDATETIME_FIELD_NAME);
    match last_update_time_t {
        None => {}
        Some(last_update_time) => {
            user.set_last_update_time(&from_iso8601(last_update_time.as_s().unwrap()));
        }
    }
    let status_t = doc.get(STATUS_FIELD_NAME).unwrap().as_s().unwrap();
    let aux = UserStatus::parse(status_t);
    match aux {
        Some(ut) => {
            user.set_status(&ut);
        }
        None => {}
    }
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
        hash_length: 32,
    };

    let hash = argon2::hash_encoded(text.as_bytes(), salt.as_bytes(), &config).unwrap();

    Ok(hash)
}

fn cypher_check(text_to_check: &String, already_ciphered: &String, key: &String) -> ResultE<bool> {
    let matches = argon2::verify_encoded_ext(
        &already_ciphered,
        text_to_check.as_bytes(),
        key.as_bytes(),
        b"",
    )
    .unwrap();

    match matches {
        true => Ok(true),
        _ => Ok(false),
    }
}
