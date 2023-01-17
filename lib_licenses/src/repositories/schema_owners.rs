

use aws_sdk_dynamodb::{model::{
    AttributeDefinition, KeySchemaElement, KeyType, ScalarAttributeType, BillingMode
},  Error};

pub const OWNERS_TABLE_NAME: &str = "truly_owners";
pub const USER_ID_FIELD: &str = "userId";
pub const ASSET_ID_FIELD: &str = "assetId";

//pub async fn create_schema_owners(conf: &Config) -> Result<(),Error> {
pub async fn create_schema_owners(client: &aws_sdk_dynamodb::Client) -> Result<(),Error> {

    let ad1 = AttributeDefinition::builder()
        .attribute_name(ASSET_ID_FIELD)
        .attribute_type(ScalarAttributeType::S)
        .build();
    let ad2 = AttributeDefinition::builder()
        .attribute_name(USER_ID_FIELD)
        .attribute_type(ScalarAttributeType::S)
        .build();

    let ks1= KeySchemaElement::builder()
        .attribute_name(USER_ID_FIELD)
        .key_type(KeyType::Hash)
        .build();
    let ks2= KeySchemaElement::builder()
        .attribute_name(ASSET_ID_FIELD)
        .key_type(KeyType::Range)
        .build();
    
    

    //let client = Client::new(conf.aws_config());

    client
        .create_table()
        .table_name(OWNERS_TABLE_NAME)
        .key_schema(ks1)
        .key_schema(ks2)
        .attribute_definitions(ad1)
        .attribute_definitions(ad2)
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await?;
    Ok(())

}

//pub async fn delete_schema_owners(conf: &Config) -> Result<(),Error> {
pub async fn delete_schema_owners(client: &aws_sdk_dynamodb::Client) -> Result<(),Error> {
    //let client = Client::new(conf.aws_config());

    client.delete_table().table_name(OWNERS_TABLE_NAME).send().await?;

    Ok(())
}
