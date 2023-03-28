

use aws_sdk_dynamodb::{model::{
    AttributeDefinition, KeySchemaElement, KeyType, ScalarAttributeType, BillingMode, GlobalSecondaryIndex, Projection, ProjectionType,
},  Error};

pub const CONTRACT_TABLE_NAME: &str = "truly_contract";
pub const CONTRACT_ID_FIELD_PK: &str = "contract_id";
pub const CONTRACT_BLOCKCHAIN_FIELD: &str = "blockchain";
pub const CONTRACT_BLOCKCHAIN_INDEX: &str = "blockchain_index";
pub const CONTRACT_STATUS_FIELD_NAME: &str = "status";

pub async fn create_schema_transactions(client: &aws_sdk_dynamodb::Client) -> Result<(),Error> {


    let id_ad = AttributeDefinition::builder()
        .attribute_name(CONTRACT_ID_FIELD_PK)
        .attribute_type(ScalarAttributeType::N)
        .build();

    let blockchain_ad = AttributeDefinition::builder()
        .attribute_name(CONTRACT_BLOCKCHAIN_FIELD)
        .attribute_type(ScalarAttributeType::S)
        .build();


    let ks1 = KeySchemaElement::builder()
        .attribute_name(CONTRACT_ID_FIELD_PK)
        .key_type(KeyType::Hash)
        .build();

    let second_index = GlobalSecondaryIndex::builder()
        .index_name(CONTRACT_BLOCKCHAIN_INDEX)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name(CONTRACT_BLOCKCHAIN_FIELD)
                .key_type(KeyType::Hash)
                .build(),
        )
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name(CONTRACT_STATUS_FIELD_NAME)
                .key_type(KeyType::Range)
                .build(),
        )
        .projection(
            Projection::builder()
                .projection_type(ProjectionType::All) //due to is a very short table and very limitted rows
                .build(),
        )
        .build();


    let op = client
        .create_table()
        .table_name(CONTRACT_TABLE_NAME)
        .key_schema(ks1)
        .global_secondary_indexes(second_index)
        .attribute_definitions(id_ad)
        .attribute_definitions(blockchain_ad)
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await;
    match op {
        Err(e)=> {
            return Err(e.into())
        },
        Ok(_)=> Ok(())
    } 

}
pub async fn delete_schema_transactions(client: &aws_sdk_dynamodb::Client) -> Result<(),Error> {

    client.delete_table().table_name(CONTRACT_TABLE_NAME).send().await?;

    Ok(())
}

