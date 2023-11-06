use crate::SERVICE;
use async_trait::async_trait;
use aws_sdk_dynamodb::types::{
    builders::StreamSpecificationBuilder, AttributeDefinition, BillingMode, GlobalSecondaryIndex,
    KeySchemaElement, KeyType, Projection, ProjectionType, ScalarAttributeType, StreamViewType,
    Tag,
};
use lib_config::{
    config::Config,
    environment::PROD_ENV,
    result::ResultE,
    schema::{Schema, schema_exists, wait_until_schema_is_active},
    constants::{
        VALUE_PROJECT, API_DOMAIN, TAG_PROJECT, TAG_SERVICE, TAG_ENVIRONMENT
    }
};

lazy_static! {
    pub static ref USERS_TABLE_NAME: String = format!("{}_{}_{}_users", VALUE_PROJECT, API_DOMAIN, SERVICE);
}
pub const USERID_FIELD_NAME_PK: &str = "userID";

lazy_static! {
    pub static ref LOGIN_EMAIL_TABLE_NAME: String = format!("{}_{}_{}_login_emails", VALUE_PROJECT, API_DOMAIN, SERVICE);
}
pub const LOGIN_EMAIL_FIELD_NAME: &str = "email";
pub const LOGIN_EMAIL_INDEX: &str = "index_email";

lazy_static! {
    pub static ref LOGIN_DEVICE_TABLE_NAME: String = format!("{}_{}_{}_login_devices", VALUE_PROJECT,API_DOMAIN, SERVICE);
}
pub const LOGIN_DEVICE_FIELD_NAME: &str = "device";
pub const LOGIN_DEVICE_INDEX: &str = "index_device";

lazy_static! {
    pub static ref LOGIN_WALLET_TABLE_NAME: String = format!("{}_{}_{}_login_wallets", VALUE_PROJECT,API_DOMAIN, SERVICE);
}
pub const LOGIN_WALLET_FIELD_NAME: &str = "wallet";
pub const LOGIN_WALLET_INDEX: &str = "index_wallet";


pub struct UserSchema;
#[async_trait]
impl Schema for UserSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {

        let exist = schema_exists(config, USERS_TABLE_NAME.as_str()).await?;
        if exist{
            return Ok(())
        }

        // main users' table
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());

        let user_id_ad = AttributeDefinition::builder()
            .attribute_name(USERID_FIELD_NAME_PK)
            .attribute_type(ScalarAttributeType::S)
            .build()
            .unwrap();

        let pk = KeySchemaElement::builder()
            .attribute_name(USERID_FIELD_NAME_PK)
            .key_type(KeyType::Hash)
            .build()
            .unwrap();

        client
            .create_table()
            .table_name(USERS_TABLE_NAME.clone())
            .key_schema(pk)
            .attribute_definitions(user_id_ad)
            .billing_mode(BillingMode::PayPerRequest)
            .stream_specification(
                StreamSpecificationBuilder::default()
                    .stream_enabled(true)
                    .stream_view_type(StreamViewType::NewAndOldImages)
                    .build()
                    .unwrap(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_ENVIRONMENT.to_string()))
                    .set_value(Some(config.env_vars().environment().unwrap()))
                    .build()
                    .unwrap(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_PROJECT.to_string()))
                    .set_value(Some(VALUE_PROJECT.to_string()))
                    .build()
                    .unwrap(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_SERVICE.to_string()))
                    .set_value(Some(API_DOMAIN.to_string()))
                    .build()
                    .unwrap(),
            )
            .deletion_protection_enabled(if config.env_vars().environment().unwrap() == PROD_ENV {
                true
            } else {
                false
            })
            .send()
            .await?;

        wait_until_schema_is_active(config, USERS_TABLE_NAME.as_str()).await?;
        Ok(())
    }

    async fn delete_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());
        client
            .delete_table()
            .table_name(USERS_TABLE_NAME.clone())
            .send()
            .await?;

        Ok(())
    }
}

pub struct LoginDeviceSchema;
#[async_trait]
impl Schema for LoginDeviceSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
        
        let exist = schema_exists(config, LOGIN_DEVICE_TABLE_NAME.as_str()).await?;
        if exist{
            return Ok(())
        }
        // main users' table
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());

        let device_user_id_ad = AttributeDefinition::builder()
            .attribute_name(USERID_FIELD_NAME_PK)
            .attribute_type(ScalarAttributeType::S)
            .build()
            .unwrap();
        let device_ad = AttributeDefinition::builder()
            .attribute_name(LOGIN_DEVICE_FIELD_NAME)
            .attribute_type(ScalarAttributeType::S)
            .build()
            .unwrap();
        let device_pk = KeySchemaElement::builder()
            .attribute_name(USERID_FIELD_NAME_PK)
            .key_type(KeyType::Hash)
            .build()
            .unwrap();
        let second_index_by_device = GlobalSecondaryIndex::builder()
            .index_name(LOGIN_DEVICE_INDEX)
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name(LOGIN_DEVICE_FIELD_NAME)
                    .key_type(KeyType::Hash)
                    .build()
                    .unwrap(),
            )
            .projection(
                Projection::builder()
                    .projection_type(ProjectionType::KeysOnly)
                    .build(),
            )
            .build()
            .unwrap();
        client
            .create_table()
            .table_name(LOGIN_DEVICE_TABLE_NAME.clone())
            .key_schema(device_pk)
            .global_secondary_indexes(second_index_by_device)
            .attribute_definitions(device_ad)
            .attribute_definitions(device_user_id_ad)
            .billing_mode(BillingMode::PayPerRequest)
            .stream_specification(
                StreamSpecificationBuilder::default()
                    .stream_enabled(true)
                    .stream_view_type(StreamViewType::NewAndOldImages)
                    .build()
                    .unwrap(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_ENVIRONMENT.to_string()))
                    .set_value(Some(config.env_vars().environment().unwrap()))
                    .build()
                    .unwrap(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_PROJECT.to_string()))
                    .set_value(Some(VALUE_PROJECT.to_string()))
                    .build()
                    .unwrap(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_SERVICE.to_string()))
                    .set_value(Some(API_DOMAIN.to_string()))
                    .build()
                    .unwrap(),
            )
            .send()
            .await?;

        wait_until_schema_is_active(config, LOGIN_DEVICE_TABLE_NAME.as_str()).await?;
        Ok(())
    }

    async fn delete_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());

        client
            .delete_table()
            .table_name(LOGIN_DEVICE_TABLE_NAME.clone())
            .send()
            .await?;

        Ok(())
    }
}

pub struct LoginEmailSchema;
#[async_trait]
impl Schema for LoginEmailSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {

        let exist = schema_exists(config, LOGIN_EMAIL_TABLE_NAME.as_str()).await?;
        if exist{
            return Ok(())
        }
        // main users' table
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());

        // Login table emails

        let email_user_id_ad = AttributeDefinition::builder()
            .attribute_name(USERID_FIELD_NAME_PK)
            .attribute_type(ScalarAttributeType::S)
            .build()
            .unwrap();
        let email_ad = AttributeDefinition::builder()
            .attribute_name(LOGIN_EMAIL_FIELD_NAME)
            .attribute_type(ScalarAttributeType::S)
            .build()
            .unwrap();
        let email_pk = KeySchemaElement::builder()
            .attribute_name(USERID_FIELD_NAME_PK)
            .key_type(KeyType::Hash)
            .build()
            .unwrap();
        let second_index_by_email = GlobalSecondaryIndex::builder()
            .index_name(LOGIN_EMAIL_INDEX)
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name(LOGIN_EMAIL_FIELD_NAME)
                    .key_type(KeyType::Hash)
                    .build()
                    .unwrap(),
            )
            .projection(
                Projection::builder()
                    .projection_type(ProjectionType::All)
                    .build(),
            )
            .build()
            .unwrap();
        client
            .create_table()
            .table_name(LOGIN_EMAIL_TABLE_NAME.clone())
            .key_schema(email_pk)
            .global_secondary_indexes(second_index_by_email)
            .attribute_definitions(email_ad)
            .attribute_definitions(email_user_id_ad)
            .billing_mode(BillingMode::PayPerRequest)
            .stream_specification(
                StreamSpecificationBuilder::default()
                    .stream_enabled(true)
                    .stream_view_type(StreamViewType::NewAndOldImages)
                    .build()
                    .unwrap(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_ENVIRONMENT.to_string()))
                    .set_value(Some(config.env_vars().environment().unwrap()))
                    .build()
                    .unwrap(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_PROJECT.to_string()))
                    .set_value(Some(VALUE_PROJECT.to_string()))
                    .build()
                    .unwrap(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_SERVICE.to_string()))
                    .set_value(Some(API_DOMAIN.to_string()))
                    .build()
                    .unwrap(),
            )
            .send()
            .await?;

        wait_until_schema_is_active(config, LOGIN_EMAIL_TABLE_NAME.as_str()).await?;
        Ok(())
    }

    async fn delete_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());

        client
            .delete_table()
            .table_name(LOGIN_EMAIL_TABLE_NAME.clone())
            .send()
            .await?;

        Ok(())
    }
}

pub struct LoginWalletSchema;
#[async_trait]
impl Schema for LoginWalletSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
        
        let exist = schema_exists(config, LOGIN_WALLET_TABLE_NAME.as_str()).await?;
        if exist{
            return Ok(())
        }

        let client = aws_sdk_dynamodb::Client::new(config.aws_config());

        let wallet_user_id_ad = AttributeDefinition::builder()
            .attribute_name(USERID_FIELD_NAME_PK)
            .attribute_type(ScalarAttributeType::S)
            .build()
            .unwrap();
        let wallet_ad = AttributeDefinition::builder()
            .attribute_name(LOGIN_WALLET_FIELD_NAME)
            .attribute_type(ScalarAttributeType::S)
            .build()
            .unwrap();
        let wallet_pk = KeySchemaElement::builder()
            .attribute_name(USERID_FIELD_NAME_PK)
            .key_type(KeyType::Hash)
            .build()
            .unwrap();
        let second_index_by_wallet = GlobalSecondaryIndex::builder()
            .index_name(LOGIN_WALLET_INDEX)
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name(LOGIN_WALLET_FIELD_NAME)
                    .key_type(KeyType::Hash)
                    .build()
                    .unwrap(),
            )
            .projection(
                Projection::builder()
                    .projection_type(ProjectionType::KeysOnly)
                    .build(),
            )
            .build()
            .unwrap();

        client
            .create_table()
            .table_name(LOGIN_WALLET_TABLE_NAME.clone())
            .key_schema(wallet_pk)
            .global_secondary_indexes(second_index_by_wallet)
            .attribute_definitions(wallet_ad)
            .attribute_definitions(wallet_user_id_ad)
            .billing_mode(BillingMode::PayPerRequest)
            .stream_specification(
                StreamSpecificationBuilder::default()
                    .stream_enabled(true)
                    .stream_view_type(StreamViewType::NewAndOldImages)
                    .build()
                    .unwrap(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_ENVIRONMENT.to_string()))
                    .set_value(Some(config.env_vars().environment().unwrap()))
                    .build()
                    .unwrap(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_PROJECT.to_string()))
                    .set_value(Some(VALUE_PROJECT.to_string()))
                    .build()
                    .unwrap(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_SERVICE.to_string()))
                    .set_value(Some(API_DOMAIN.to_string()))
                    .build()
                    .unwrap(),
            )
            .send()
            .await?;
        wait_until_schema_is_active(config, LOGIN_WALLET_TABLE_NAME.as_str()).await?;
        Ok(())
    }

    async fn delete_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());

        client
            .delete_table()
            .table_name(LOGIN_WALLET_TABLE_NAME.clone())
            .send()
            .await?;

        Ok(())
    }
}

pub struct UserAllSchema;
#[async_trait]
impl Schema for UserAllSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
        UserSchema::create_schema(config).await?;
        LoginDeviceSchema::create_schema(config).await?;
        LoginEmailSchema::create_schema(config).await?;
        LoginWalletSchema::create_schema(config).await?;
        Ok(())
    }
    async fn delete_schema(config: &Config) -> ResultE<()> {
        UserSchema::delete_schema(config).await?;
        LoginDeviceSchema::delete_schema(config).await?;
        LoginEmailSchema::delete_schema(config).await?;
        LoginWalletSchema::delete_schema(config).await?;
        Ok(())
    }
}




