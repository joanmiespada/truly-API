

use aws_sdk_dynamodb::{model::{
    AttributeDefinition, KeySchemaElement, KeyType, ScalarAttributeType, BillingMode, ProjectionType, Projection, GlobalSecondaryIndex
},  Error};

pub const OWNERS_TABLE_NAME: &str = "truly_owners";
pub const OWNER_USER_ID_FIELD_PK: &str = "userId";
pub const OWNER_ASSET_ID_FIELD_PK: &str = "assetId";
pub const OWNERS_USER_ID_INDEX: &str = "user_id_index";
pub const OWNERS_ASSET_ID_INDEX: &str = "asset_id_index";

//pub async fn create_schema_owners(conf: &Config) -> Result<(),Error> {
pub async fn create_schema_owners(client: &aws_sdk_dynamodb::Client) -> Result<(),Error> {

    let ad1 = AttributeDefinition::builder()
        .attribute_name(OWNER_USER_ID_FIELD_PK)
        .attribute_type(ScalarAttributeType::S)
        .build();
    let ad2 = AttributeDefinition::builder()
        .attribute_name(OWNER_ASSET_ID_FIELD_PK)
        .attribute_type(ScalarAttributeType::S)
        .build();
    

    let ks1= KeySchemaElement::builder()
        .attribute_name(OWNER_USER_ID_FIELD_PK)
        .key_type(KeyType::Hash)
        .build();
    let ks2= KeySchemaElement::builder()
        .attribute_name(OWNER_ASSET_ID_FIELD_PK)
        .key_type(KeyType::Range)
        .build();

    let second_index_by_user = GlobalSecondaryIndex::builder()
        .index_name(OWNERS_USER_ID_INDEX)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name( OWNER_USER_ID_FIELD_PK)
                .key_type(KeyType::Hash)
                .build(),
        )
        .projection(
            Projection::builder()
                .projection_type(ProjectionType::All)
                .build(),
        )
        .build(); 
    let second_index_by_asset = GlobalSecondaryIndex::builder()
        .index_name(OWNERS_ASSET_ID_INDEX)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name( OWNER_ASSET_ID_FIELD_PK)
                .key_type(KeyType::Hash)
                .build(),
        )
        .projection(
            Projection::builder()
                .projection_type(ProjectionType::All)
                .build(),
        )
        .build();

    //let client = Client::new(conf.aws_config());

    client
        .create_table()
        .table_name(OWNERS_TABLE_NAME)
        .key_schema(ks1)
        .key_schema(ks2)
        .global_secondary_indexes(second_index_by_user )
        .global_secondary_indexes(second_index_by_asset )
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
