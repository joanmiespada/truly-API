use aws_sdk_dynamodb::{
    model::{
        AttributeDefinition, BillingMode, GlobalSecondaryIndex, KeySchemaElement, KeyType,
        Projection, ProjectionType, ScalarAttributeType,
    },
    Error,
};

pub const ASSETS_TABLE_NAME: &str = "truly_assets";
pub const ASSETS_TABLE_INDEX_ID: &str = "assetId";
pub const ASSET_ID_FIELD_PK: &str = "assetId";
pub const URL_FIELD_NAME: &str = "uri";
pub const URL_INDEX_NAME: &str = "url_index";

pub const ASSET_TREE_TABLE_NAME: &str = "truly_assets_tree";
pub const ASSET_TREE_SON_ID_FIELD_PK: &str = "son_id";
pub const ASSET_TREE_FATHER_ID_FIELD_PK: &str = "father_id";
pub const ASSET_TREE_FATHER_INDEX: &str = "father_index";

pub const SHORTER_TABLE_NAME: &str = "truly_assets_shorter";
pub const SHORTER_ASSET_ID_FIELD: &str = "asset_id";
pub const SHORTER_FIELD_PK: &str = "shorter";
pub const SHORTER_ASSET_INEX: &str = "shorter_index";

async fn create_schema_assets(client: &aws_sdk_dynamodb::Client) -> Result<(), Error> {
    let asset_ad = AttributeDefinition::builder()
        .attribute_name(ASSET_ID_FIELD_PK)
        .attribute_type(ScalarAttributeType::S)
        .build();
    let url_ad = AttributeDefinition::builder()
        .attribute_name(URL_FIELD_NAME)
        .attribute_type(ScalarAttributeType::S)
        .build();

    let ks = KeySchemaElement::builder()
        .attribute_name(ASSET_ID_FIELD_PK)
        .key_type(KeyType::Hash)
        .build();

    let second_index = GlobalSecondaryIndex::builder()
        .index_name(URL_INDEX_NAME)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name(URL_FIELD_NAME)
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
        .table_name(ASSETS_TABLE_NAME)
        .key_schema(ks)
        .global_secondary_indexes(second_index)
        .attribute_definitions(asset_ad)
        .attribute_definitions(url_ad)
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await?;
    Ok(())
}
async fn delete_schema_assets(client: &aws_sdk_dynamodb::Client) -> Result<(), Error> {
    client
        .delete_table()
        .table_name(ASSETS_TABLE_NAME)
        .send()
        .await?;

    Ok(())
}

async fn create_schema_assets_tree(client: &aws_sdk_dynamodb::Client) -> Result<(), Error> {
    let ad1 = AttributeDefinition::builder()
        .attribute_name(ASSET_TREE_SON_ID_FIELD_PK)
        .attribute_type(ScalarAttributeType::S)
        .build();

    let ad2 = AttributeDefinition::builder()
        .attribute_name(ASSET_TREE_FATHER_ID_FIELD_PK)
        .attribute_type(ScalarAttributeType::S)
        .build();

    let ks1 = KeySchemaElement::builder()
        .attribute_name(ASSET_TREE_SON_ID_FIELD_PK)
        .key_type(KeyType::Hash)
        .build();
    //let ks2= KeySchemaElement::builder()
    //    .attribute_name(ASSET_TREE_FATHER_ID_FIELD_PK)
    //    .key_type(KeyType::Range)
    //    .build();

    let second_index = GlobalSecondaryIndex::builder()
        .index_name(ASSET_TREE_FATHER_INDEX)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name(ASSET_TREE_FATHER_ID_FIELD_PK)
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
        .table_name(ASSET_TREE_TABLE_NAME)
        .key_schema(ks1)
        .global_secondary_indexes(second_index)
        .attribute_definitions(ad1)
        .attribute_definitions(ad2)
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await?;

    Ok(())
}
async fn delete_schema_assets_tree(client: &aws_sdk_dynamodb::Client) -> Result<(), Error> {
    client
        .delete_table()
        .table_name(ASSET_TREE_TABLE_NAME)
        .send()
        .await?;

    Ok(())
}

async fn create_schema_shorters(client: &aws_sdk_dynamodb::Client) -> Result<(), Error> {
    let asset_ad = AttributeDefinition::builder()
        .attribute_name(SHORTER_ASSET_ID_FIELD)
        .attribute_type(ScalarAttributeType::S)
        .build();
    let shorter_ad = AttributeDefinition::builder()
        .attribute_name(SHORTER_FIELD_PK)
        .attribute_type(ScalarAttributeType::S)
        .build();

    let ks = KeySchemaElement::builder()
        .attribute_name(SHORTER_FIELD_PK)
        .key_type(KeyType::Hash)
        .build();

    let second_index = GlobalSecondaryIndex::builder()
        .index_name(SHORTER_ASSET_INEX)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name(SHORTER_ASSET_ID_FIELD)
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
        .table_name(SHORTER_TABLE_NAME)
        .key_schema(ks)
        .global_secondary_indexes(second_index)
        .attribute_definitions(asset_ad)
        .attribute_definitions(shorter_ad)
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await?;
    Ok(())
}
async fn delete_schema_shorters(client: &aws_sdk_dynamodb::Client) -> Result<(), Error> {
    client
        .delete_table()
        .table_name(SHORTER_TABLE_NAME)
        .send()
        .await?;

    Ok(())
}

pub async fn create_schema_assets_all(client: &aws_sdk_dynamodb::Client) -> Result<(), Error> {
    create_schema_assets(client).await?;
    create_schema_assets_tree(client).await?;
    create_schema_shorters(client).await?;
    Ok(())
}

pub async fn delete_schema_assets_all(client: &aws_sdk_dynamodb::Client) -> Result<(), Error> {
    delete_schema_assets(client).await?;
    delete_schema_assets_tree(client).await?;
    delete_schema_shorters(client).await?;
    Ok(())
}
