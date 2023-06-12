use aws_sdk_dynamodb::{
    types::{AttributeDefinition, BillingMode, KeySchemaElement, KeyType, ScalarAttributeType},
    Error,
};

pub const BLOCKCHAIN_TABLE_NAME: &str = "truly_blockchain";
pub const BLOCKCHAIN_ID_FIELD_PK: &str = "blockchain_id";

pub async fn create_schema_blockchains(client: &aws_sdk_dynamodb::Client) -> Result<(), Error> {
    let id_ad = AttributeDefinition::builder()
        .attribute_name(BLOCKCHAIN_ID_FIELD_PK)
        .attribute_type(ScalarAttributeType::S)
        .build();

    let ks1 = KeySchemaElement::builder()
        .attribute_name(BLOCKCHAIN_ID_FIELD_PK)
        .key_type(KeyType::Hash)
        .build();

    let op = client
        .create_table()
        .table_name(BLOCKCHAIN_TABLE_NAME)
        .key_schema(ks1)
        .attribute_definitions(id_ad)
        .billing_mode(BillingMode::PayPerRequest);
    let op = op.send().await;
    match op {
        Err(e) => return Err(e.into()),
        Ok(_) => Ok(()),
    }
}
pub async fn delete_schema_blockchains(client: &aws_sdk_dynamodb::Client) -> Result<(), Error> {
    client
        .delete_table()
        .table_name(BLOCKCHAIN_TABLE_NAME)
        .send()
        .await?;

    Ok(())
}
