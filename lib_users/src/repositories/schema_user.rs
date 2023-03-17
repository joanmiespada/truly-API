

use aws_sdk_dynamodb::{model::{
    AttributeDefinition, KeySchemaElement, KeyType, ScalarAttributeType, BillingMode, GlobalSecondaryIndex, Projection, ProjectionType 
},  Error};

pub const USERS_TABLE_NAME: &str = "truly_users";
pub const USERID_FIELD_NAME_PK: &str = "userID";

pub const LOGIN_EMAIL_TABLE_NAME: &str = "truly_login_emails";
pub const LOGIN_EMAIL_FIELD_NAME_PK: &str = "email";
pub const LOGIN_EMAIL_USERID_INDEX: &str = "user_id_index_email";

pub const LOGIN_DEVICE_TABLE_NAME: &str = "truly_login_devices";
pub const LOGIN_DEVICE_FIELD_NAME_PK: &str = "device";
pub const LOGIN_DEVICE_USERID_INDEX: &str = "user_id_index_device";

pub const LOGIN_WALLET_TABLE_NAME: &str = "truly_login_wallet";
pub const LOGIN_WALLET_FIELD_NAME_PK: &str = "wallet";
pub const LOGIN_WALLET_USERID_INDEX: &str = "user_id_index_wallet";

pub const  USERID_FIELD_NAME: &str = "user_id_foreign_key";

pub async fn create_schema_users(client: &aws_sdk_dynamodb::Client) -> Result<(),Error> {

    // main users' table

    let user_id_ad = AttributeDefinition::builder()
        .attribute_name(USERID_FIELD_NAME_PK)
        .attribute_type(ScalarAttributeType::S)
        .build();
    
    let pk = KeySchemaElement::builder()
        .attribute_name(USERID_FIELD_NAME_PK )
        .key_type(KeyType::Hash)
        .build();

    client
        .create_table()
        .table_name(USERS_TABLE_NAME)
        .key_schema(pk)
        .attribute_definitions(user_id_ad)
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await?;

    // Login table emails
    
    let email_id_ad = AttributeDefinition::builder()
        .attribute_name( LOGIN_EMAIL_FIELD_NAME_PK)
        .attribute_type(ScalarAttributeType::S)
        .build();
    let email_user_id_ad = AttributeDefinition::builder()
        .attribute_name( USERID_FIELD_NAME )
        .attribute_type(ScalarAttributeType::S)
        .build();
    let email_pk = KeySchemaElement::builder()
        .attribute_name(LOGIN_EMAIL_FIELD_NAME_PK)
        .key_type(KeyType::Hash)
        .build();
    let second_index_by_email = GlobalSecondaryIndex::builder()
        .index_name(LOGIN_EMAIL_USERID_INDEX)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name( USERID_FIELD_NAME )
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
        .table_name( LOGIN_EMAIL_TABLE_NAME)
        .key_schema(email_pk)
        .global_secondary_indexes(second_index_by_email)
        .attribute_definitions(email_id_ad)
        .attribute_definitions(email_user_id_ad)
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await?;

    // Login table devices
    
    let device_id_ad = AttributeDefinition::builder()
        .attribute_name( LOGIN_DEVICE_FIELD_NAME_PK )
        .attribute_type(ScalarAttributeType::S)
        .build();
    let device_user_id_ad = AttributeDefinition::builder()
        .attribute_name( USERID_FIELD_NAME )
        .attribute_type(ScalarAttributeType::S)
        .build();
    let device_pk = KeySchemaElement::builder()
        .attribute_name( LOGIN_DEVICE_FIELD_NAME_PK)
        .key_type(KeyType::Hash)
        .build();
    let second_index_by_device = GlobalSecondaryIndex::builder()
        .index_name(LOGIN_DEVICE_USERID_INDEX)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name( USERID_FIELD_NAME )
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
        .table_name( LOGIN_DEVICE_TABLE_NAME )
        .key_schema(device_pk)
        .global_secondary_indexes(second_index_by_device)
        .attribute_definitions(device_id_ad)
        .attribute_definitions(device_user_id_ad)
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await?;

    // Login table wallet
    
    let wallet_id_ad = AttributeDefinition::builder()
        .attribute_name( LOGIN_WALLET_FIELD_NAME_PK )
        .attribute_type(ScalarAttributeType::S)
        .build();
    let wallet_user_id_ad = AttributeDefinition::builder()
        .attribute_name( USERID_FIELD_NAME )
        .attribute_type(ScalarAttributeType::S)
        .build();
    let wallet_pk = KeySchemaElement::builder()
        .attribute_name( LOGIN_WALLET_FIELD_NAME_PK)
        .key_type(KeyType::Hash)
        .build();
    let second_index_by_wallet = GlobalSecondaryIndex::builder()
        .index_name(LOGIN_WALLET_USERID_INDEX)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name( USERID_FIELD_NAME )
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
        .table_name( LOGIN_WALLET_TABLE_NAME )
        .key_schema(wallet_pk)
        .global_secondary_indexes(second_index_by_wallet)
        .attribute_definitions(wallet_id_ad)
        .attribute_definitions(wallet_user_id_ad)
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await?;
   Ok(()) 

}

pub async fn delete_schema_users(client: &aws_sdk_dynamodb::Client) -> Result<(),Error> {

    client.delete_table().table_name(USERS_TABLE_NAME).send().await?;

    Ok(())
}

