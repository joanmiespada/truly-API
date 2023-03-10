

use aws_sdk_dynamodb::{model::{
    AttributeDefinition, KeySchemaElement, KeyType, ScalarAttributeType, BillingMode, GlobalSecondaryIndex, Projection, ProjectionType,
},  Error};

pub const BLOCKCHAIN_TX_TABLE_NAME: &str = "truly_blockchain_txs";
pub const TX_ASSET_ID_FIELD_PK: &str = "assetId";
pub const TX_TIMESTAMP_PK: &str = "timestamp";
pub const TX_FIELD: &str = "tx";
pub const TX_INDEX_NAME: &str = "tx_index";

pub async fn create_schema_assets(client: &aws_sdk_dynamodb::Client) -> Result<(),Error> {


    let asset_ad = AttributeDefinition::builder()
        .attribute_name(TX_ASSET_ID_FIELD_PK)
        .attribute_type(ScalarAttributeType::S)
        .build();
    let tx_ad = AttributeDefinition::builder()
        .attribute_name( TX_TIMESTAMP_PK)
        .attribute_type(ScalarAttributeType::S)
        .build();
    
    let ks1 = KeySchemaElement::builder()
        .attribute_name(TX_ASSET_ID_FIELD_PK)
        .key_type(KeyType::Hash)
        .build();
    let ks2 = KeySchemaElement::builder()
        .attribute_name(TX_TIMESTAMP_PK)
        .key_type(KeyType::Range)
        .build();

    let second_index = GlobalSecondaryIndex::builder()
        .index_name(TX_INDEX_NAME)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name(TX_FIELD)
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
        .table_name(BLOCKCHAIN_TX_TABLE_NAME)
        .key_schema(ks1)
        .key_schema(ks2)
        .global_secondary_indexes(second_index)
        .attribute_definitions(asset_ad)
        .attribute_definitions(tx_ad)
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await?;
   Ok(()) 

}
pub async fn delete_schema_assets(client: &aws_sdk_dynamodb::Client) -> Result<(),Error> {

    client.delete_table().table_name(BLOCKCHAIN_TX_TABLE_NAME).send().await?;

    Ok(())
}

