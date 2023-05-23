use aws_sdk_dynamodb::{
    types::{
        AttributeDefinition, BillingMode, GlobalSecondaryIndex, KeySchemaElement, KeyType,
        Projection, ProjectionType, ScalarAttributeType,
    },
    Error,
};

pub const KEYPAIRS_TABLE_NAME: &str = "truly_users_keypairs";
pub const KEYPAIRS_USER_ID_FIELD_PK: &str = "userId";
pub const KEYPAIRS_ADDRESS_FIELD: &str = "address";
pub const KEYPAIRS_ADDRESS_INDEX_NAME: &str = "address_index";
pub const KEYPAIRS_PUBLIC_FIELD: &str = "public_key_enc";
pub const KEYPAIRS_PRIVATE_FIELD: &str = "private_key_enc";

//pub async fn create_schema_owners(conf: &Config) -> Result<(),Error> {
pub async fn create_schema_keypairs(client: &aws_sdk_dynamodb::Client) -> Result<(), Error> {
    let ad1 = AttributeDefinition::builder()
        .attribute_name(KEYPAIRS_USER_ID_FIELD_PK)
        .attribute_type(ScalarAttributeType::S)
        .build();
    let ad2 = AttributeDefinition::builder()
        .attribute_name(KEYPAIRS_ADDRESS_FIELD)
        .attribute_type(ScalarAttributeType::S)
        .build();

    let ks_by_user_id = KeySchemaElement::builder()
        .attribute_name(KEYPAIRS_USER_ID_FIELD_PK)
        .key_type(KeyType::Hash)
        .build();

    let second_index = GlobalSecondaryIndex::builder()
        .index_name(KEYPAIRS_ADDRESS_INDEX_NAME)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name(KEYPAIRS_ADDRESS_FIELD)
                .key_type(KeyType::Hash)
                .build(),
        )
        .projection(
            Projection::builder()
                .projection_type(ProjectionType::KeysOnly)
                .build(),
        )
        .build();

    //let client = Client::new(conf.aws_config());

    client
        .create_table()
        .table_name(KEYPAIRS_TABLE_NAME)
        .key_schema(ks_by_user_id)
        .global_secondary_indexes(second_index)
        .attribute_definitions(ad1)
        .attribute_definitions(ad2)
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await?;
    Ok(())
}

//pub async fn delete_schema_owners(conf: &Config) -> Result<(),Error> {
pub async fn delete_schema_keypairs(client: &aws_sdk_dynamodb::Client) -> Result<(), Error> {
    //let client = Client::new(conf.aws_config());

    client
        .delete_table()
        .table_name(KEYPAIRS_TABLE_NAME)
        .send()
        .await?;

    Ok(())
}
