

use aws_sdk_dynamodb::{model::{
    AttributeDefinition, KeySchemaElement, KeyType, ScalarAttributeType, BillingMode, Projection, ProjectionType, GlobalSecondaryIndex,
},  Error};

pub const USERS_TABLE_NAME: &str = "truly_users";
pub const USERS_TABLE_INDEX_EMAIL: &str = "email";
pub const USERS_TABLE_INDEX_DEVICE: &str = "device";
pub const DEVICE_FIELD_NAME: &str = "device";
pub const USERID_FIELD_NAME_PK: &str = "userID";
pub const EMAIL_FIELD_NAME: &str = "email";

pub async fn create_schema_users(client: &aws_sdk_dynamodb::Client) -> Result<(),Error> {


    let user_id_ad = AttributeDefinition::builder()
        .attribute_name(USERID_FIELD_NAME_PK)
        .attribute_type(ScalarAttributeType::S)
        .build();
     
    let email_ad = AttributeDefinition::builder()
        .attribute_name(EMAIL_FIELD_NAME)
        .attribute_type(ScalarAttributeType::S)
        .build();   

    let device_ad = AttributeDefinition::builder()
        .attribute_name( DEVICE_FIELD_NAME)
        .attribute_type(ScalarAttributeType::S)
        .build();

    let pk = KeySchemaElement::builder()
        .attribute_name(USERID_FIELD_NAME_PK )
        .key_type(KeyType::Hash)
        .build();

    let second_index_email = GlobalSecondaryIndex::builder()
        .index_name(USERS_TABLE_INDEX_EMAIL)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name(EMAIL_FIELD_NAME)
                .key_type(KeyType::Hash)
                .build(),
        )
        .projection(
            Projection::builder()
                .projection_type(ProjectionType::KeysOnly)
                .build(),
        )
        .build();

    let second_index_device = GlobalSecondaryIndex::builder()
        .index_name(USERS_TABLE_INDEX_DEVICE)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name(DEVICE_FIELD_NAME)
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
        .table_name(USERS_TABLE_NAME)
        .key_schema(pk)
        .global_secondary_indexes(second_index_email)
        .global_secondary_indexes(second_index_device)
        .attribute_definitions(user_id_ad)
        .attribute_definitions(email_ad )
        .attribute_definitions(device_ad )
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await?;
   Ok(()) 

}

pub async fn delete_schema_users(client: &aws_sdk_dynamodb::Client) -> Result<(),Error> {

    client.delete_table().table_name(USERS_TABLE_NAME).send().await?;

    Ok(())
}

