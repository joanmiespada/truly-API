use crate::SERVICE;
use async_trait::async_trait;
use aws_sdk_dynamodb::types::{
    builders::StreamSpecificationBuilder, AttributeDefinition, BillingMode, GlobalSecondaryIndex,
    KeySchemaElement, KeyType, Projection, ProjectionType, ScalarAttributeType, StreamViewType,
    Tag,
};
use lib_config::{
    config::Config,
    environment::{
        ENV_VAR_ENVIRONMENT, ENV_VAR_PROJECT, ENV_VAR_PROJECT_LABEL, ENV_VAR_SERVICE_LABEL, PROD_ENV,
    },
    result::ResultE,
    schema::Schema,
};

pub const USERS_TABLE_NAME: &str = "truly_users";
pub const USERID_FIELD_NAME_PK: &str = "userID";

pub const LOGIN_EMAIL_TABLE_NAME: &str = "truly_login_emails";
pub const LOGIN_EMAIL_FIELD_NAME: &str = "email";
pub const LOGIN_EMAIL_INDEX: &str = "index_email";

pub const LOGIN_DEVICE_TABLE_NAME: &str = "truly_login_devices";
pub const LOGIN_DEVICE_FIELD_NAME: &str = "device";
pub const LOGIN_DEVICE_INDEX: &str = "index_device";

pub const LOGIN_WALLET_TABLE_NAME: &str = "truly_login_wallet";
pub const LOGIN_WALLET_FIELD_NAME: &str = "wallet";
pub const LOGIN_WALLET_INDEX: &str = "index_wallet";


pub struct UserSchema;
#[async_trait]
impl Schema for UserSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
        // main users' table
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());

        let user_id_ad = AttributeDefinition::builder()
            .attribute_name(USERID_FIELD_NAME_PK)
            .attribute_type(ScalarAttributeType::S)
            .build();

        let pk = KeySchemaElement::builder()
            .attribute_name(USERID_FIELD_NAME_PK)
            .key_type(KeyType::Hash)
            .build();

        client
            .create_table()
            .table_name(USERS_TABLE_NAME)
            .key_schema(pk)
            .attribute_definitions(user_id_ad)
            .billing_mode(BillingMode::PayPerRequest)
            .stream_specification(
                StreamSpecificationBuilder::default()
                    .stream_enabled(true)
                    .stream_view_type(StreamViewType::NewAndOldImages)
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(ENV_VAR_ENVIRONMENT.to_string()))
                    .set_value(Some(config.env_vars().environment().unwrap()))
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(ENV_VAR_PROJECT_LABEL.to_string()))
                    .set_value(Some(ENV_VAR_PROJECT.to_string()))
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(ENV_VAR_SERVICE_LABEL.to_string()))
                    .set_value(Some(SERVICE.to_string()))
                    .build(),
            )
            .deletion_protection_enabled(if config.env_vars().environment().unwrap() == PROD_ENV {
                true
            } else {
                false
            })
            .send()
            .await?;

        Ok(())
    }

    async fn delete_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());
        client
            .delete_table()
            .table_name(USERS_TABLE_NAME)
            .send()
            .await?;

        Ok(())
    }
}

pub struct LoginDeviceSchema;
#[async_trait]
impl Schema for LoginDeviceSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
        // main users' table
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());

        let device_user_id_ad = AttributeDefinition::builder()
            .attribute_name(USERID_FIELD_NAME_PK)
            .attribute_type(ScalarAttributeType::S)
            .build();
        let device_ad = AttributeDefinition::builder()
            .attribute_name(LOGIN_DEVICE_FIELD_NAME)
            .attribute_type(ScalarAttributeType::S)
            .build();
        let device_pk = KeySchemaElement::builder()
            .attribute_name(USERID_FIELD_NAME_PK)
            .key_type(KeyType::Hash)
            .build();
        let second_index_by_device = GlobalSecondaryIndex::builder()
            .index_name(LOGIN_DEVICE_INDEX)
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name(LOGIN_DEVICE_FIELD_NAME)
                    .key_type(KeyType::Hash)
                    .build(),
            )
            .projection(
                Projection::builder()
                    .projection_type(ProjectionType::KeysOnly)
                    .build(),
            )
            .build();
        client
            .create_table()
            .table_name(LOGIN_DEVICE_TABLE_NAME)
            .key_schema(device_pk)
            .global_secondary_indexes(second_index_by_device)
            .attribute_definitions(device_ad)
            .attribute_definitions(device_user_id_ad)
            .billing_mode(BillingMode::PayPerRequest)
            .stream_specification(
                StreamSpecificationBuilder::default()
                    .stream_enabled(true)
                    .stream_view_type(StreamViewType::NewAndOldImages)
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(ENV_VAR_ENVIRONMENT.to_string()))
                    .set_value(Some(config.env_vars().environment().unwrap()))
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(ENV_VAR_PROJECT_LABEL.to_string()))
                    .set_value(Some(ENV_VAR_PROJECT.to_string()))
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(ENV_VAR_SERVICE_LABEL.to_string()))
                    .set_value(Some(SERVICE.to_string()))
                    .build(),
            )
            .send()
            .await?;

        Ok(())
    }

    async fn delete_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());

        client
            .delete_table()
            .table_name(LOGIN_DEVICE_TABLE_NAME)
            .send()
            .await?;

        Ok(())
    }
}

pub struct LoginEmailSchema;
#[async_trait]
impl Schema for LoginEmailSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
        // main users' table
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());

        // Login table emails

        let email_user_id_ad = AttributeDefinition::builder()
            .attribute_name(USERID_FIELD_NAME_PK)
            .attribute_type(ScalarAttributeType::S)
            .build();
        let email_ad = AttributeDefinition::builder()
            .attribute_name(LOGIN_EMAIL_FIELD_NAME)
            .attribute_type(ScalarAttributeType::S)
            .build();
        let email_pk = KeySchemaElement::builder()
            .attribute_name(USERID_FIELD_NAME_PK)
            .key_type(KeyType::Hash)
            .build();
        let second_index_by_email = GlobalSecondaryIndex::builder()
            .index_name(LOGIN_EMAIL_INDEX)
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name(LOGIN_EMAIL_FIELD_NAME)
                    .key_type(KeyType::Hash)
                    .build(),
            )
            .projection(
                Projection::builder()
                    .projection_type(ProjectionType::All)
                    .build(),
            )
            .build();
        client
            .create_table()
            .table_name(LOGIN_EMAIL_TABLE_NAME)
            .key_schema(email_pk)
            .global_secondary_indexes(second_index_by_email)
            .attribute_definitions(email_ad)
            .attribute_definitions(email_user_id_ad)
            .billing_mode(BillingMode::PayPerRequest)
            .stream_specification(
                StreamSpecificationBuilder::default()
                    .stream_enabled(true)
                    .stream_view_type(StreamViewType::NewAndOldImages)
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(ENV_VAR_ENVIRONMENT.to_string()))
                    .set_value(Some(config.env_vars().environment().unwrap()))
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(ENV_VAR_PROJECT_LABEL.to_string()))
                    .set_value(Some(ENV_VAR_PROJECT.to_string()))
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(ENV_VAR_SERVICE_LABEL.to_string()))
                    .set_value(Some(SERVICE.to_string()))
                    .build(),
            )
            .send()
            .await?;

        Ok(())
    }

    async fn delete_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());

        client
            .delete_table()
            .table_name(LOGIN_EMAIL_TABLE_NAME)
            .send()
            .await?;

        Ok(())
    }
}

pub struct LoginWalletSchema;
#[async_trait]
impl Schema for LoginWalletSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());

        let wallet_user_id_ad = AttributeDefinition::builder()
            .attribute_name(USERID_FIELD_NAME_PK)
            .attribute_type(ScalarAttributeType::S)
            .build();
        let wallet_ad = AttributeDefinition::builder()
            .attribute_name(LOGIN_WALLET_FIELD_NAME)
            .attribute_type(ScalarAttributeType::S)
            .build();
        let wallet_pk = KeySchemaElement::builder()
            .attribute_name(USERID_FIELD_NAME_PK)
            .key_type(KeyType::Hash)
            .build();
        let second_index_by_wallet = GlobalSecondaryIndex::builder()
            .index_name(LOGIN_WALLET_INDEX)
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name(LOGIN_WALLET_FIELD_NAME)
                    .key_type(KeyType::Hash)
                    .build(),
            )
            .projection(
                Projection::builder()
                    .projection_type(ProjectionType::KeysOnly)
                    .build(),
            )
            .build();

        client
            .create_table()
            .table_name(LOGIN_WALLET_TABLE_NAME)
            .key_schema(wallet_pk)
            .global_secondary_indexes(second_index_by_wallet)
            .attribute_definitions(wallet_ad)
            .attribute_definitions(wallet_user_id_ad)
            .billing_mode(BillingMode::PayPerRequest)
            .stream_specification(
                StreamSpecificationBuilder::default()
                    .stream_enabled(true)
                    .stream_view_type(StreamViewType::NewAndOldImages)
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(ENV_VAR_ENVIRONMENT.to_string()))
                    .set_value(Some(config.env_vars().environment().unwrap()))
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(ENV_VAR_PROJECT_LABEL.to_string()))
                    .set_value(Some(ENV_VAR_PROJECT.to_string()))
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(ENV_VAR_SERVICE_LABEL.to_string()))
                    .set_value(Some(SERVICE.to_string()))
                    .build(),
            )
            .send()
            .await?;
        Ok(())
    }

    async fn delete_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());

        client
            .delete_table()
            .table_name(LOGIN_WALLET_TABLE_NAME)
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
