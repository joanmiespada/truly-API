use aws_sdk_dynamodb::types::{
    AttributeDefinition, BillingMode, GlobalSecondaryIndex, KeySchemaElement, KeyType, Projection,
    ProjectionType, ScalarAttributeType,
};
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::Error;

pub const LICENSES_TABLE_NAME: &str = "truly_licenses";
pub const LICENSE_ID_FIELD_PK: &str = "licenseId";
pub const LICENSE_ASSET_ID_FIELD_PK: &str = "assetId";
pub const LICENSES_ASSET_ID_INDEX: &str = "asset_id_index";
pub const LICENSES_LICENSE_ID_INDEX: &str = "license_id_index";

pub async fn create_schema_licenses(client: &Client) -> Result<(), Error> {
    let ad1 = AttributeDefinition::builder()
        .attribute_name(LICENSE_ID_FIELD_PK)
        .attribute_type(ScalarAttributeType::S)
        .build();
    let ad2 = AttributeDefinition::builder()
        .attribute_name(LICENSE_ASSET_ID_FIELD_PK)
        .attribute_type(ScalarAttributeType::S)
        .build();

    let ks1 = KeySchemaElement::builder()
        .attribute_name(LICENSE_ID_FIELD_PK)
        .key_type(KeyType::Hash)
        .build();
    let ks2 = KeySchemaElement::builder()
        .attribute_name(LICENSE_ASSET_ID_FIELD_PK)
        .key_type(KeyType::Range)
        .build();

    let second_index_by_asset = GlobalSecondaryIndex::builder()
        .index_name(LICENSES_ASSET_ID_INDEX)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name(LICENSE_ASSET_ID_FIELD_PK)
                .key_type(KeyType::Hash)
                .build(),
        )
        .projection(
            Projection::builder()
                .projection_type(ProjectionType::All)
                .build(),
        )
        .build();
    let third_index_by_asset = GlobalSecondaryIndex::builder()
        .index_name(LICENSES_LICENSE_ID_INDEX)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name(LICENSE_ID_FIELD_PK)
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
        .table_name(LICENSES_TABLE_NAME)
        .key_schema(ks1)
        .key_schema(ks2)
        .global_secondary_indexes(second_index_by_asset)
        .global_secondary_indexes(third_index_by_asset)
        .attribute_definitions(ad1)
        .attribute_definitions(ad2)
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await?;

    Ok(())
}

pub async fn delete_schema_licenses(client: &Client) -> Result<(), Error> {
    client
        .delete_table()
        .table_name(LICENSES_TABLE_NAME)
        .send()
        .await?;

    Ok(())
}
