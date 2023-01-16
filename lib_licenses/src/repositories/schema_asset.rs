

use aws_sdk_dynamodb::{model::{
    AttributeDefinition, KeySchemaElement, KeyType, ScalarAttributeType, BillingMode,
},  Error};

pub const ASSETS_TABLE_NAME: &str = "truly_assets";
pub const ASSETS_TABLE_INDEX_ID: &str = "assetId";
pub const ASSET_ID_FIELD_PK: &str = "assetId";

//pub async fn create_schema_assets(conf: &Config) -> Result<(),Error> {
pub async fn create_schema_assets(client: &aws_sdk_dynamodb::Client) -> Result<(),Error> {

    //let client = Client::new(&shared_config);

    let ad = AttributeDefinition::builder()
        .attribute_name(ASSET_ID_FIELD_PK)
        .attribute_type(ScalarAttributeType::S)
        .build();

    let ks = KeySchemaElement::builder()
        .attribute_name(ASSET_ID_FIELD_PK)
        .key_type(KeyType::Hash)
        .build();

    //let client = Client::new(conf.aws_config());

    client
        .create_table()
        .table_name(ASSETS_TABLE_NAME)
        .key_schema(ks)
        .attribute_definitions(ad)
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await?;
   Ok(()) 

}
//pub async fn delete_schema_assets(conf: &Config) -> Result<(),Error> {
pub async fn delete_schema_assets(client: &aws_sdk_dynamodb::Client) -> Result<(),Error> {
    //let client = Client::new(conf.aws_config());

    client.delete_table().table_name(ASSETS_TABLE_NAME).send().await?;

    Ok(())
}

