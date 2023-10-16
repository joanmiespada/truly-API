use async_trait::async_trait;
use aws_sdk_dynamodb::types::{
    builders::StreamSpecificationBuilder, AttributeDefinition, BillingMode, GlobalSecondaryIndex,
    KeySchemaElement, KeyType, Projection, ProjectionType, ScalarAttributeType, StreamViewType,
    Tag,
};
use lib_config::{
    config::Config,
    environment::PROD_ENV,
    result::ResultE,
    schema::Schema,
    constants::{
        VALUE_PROJECT, API_DOMAIN, TAG_PROJECT, TAG_SERVICE, TAG_ENVIRONMENT
    }
};

lazy_static! {
    pub static ref ASSETS_TABLE_NAME: String = format!("{}_{}_assets_assets", VALUE_PROJECT, API_DOMAIN );
}
pub const ASSETS_TABLE_INDEX_ID: &str = "assetId";
pub const ASSET_ID_FIELD_PK: &str = "assetId";
pub const URL_FIELD_NAME: &str = "uri";
pub const URL_INDEX_NAME: &str = "url_index";

lazy_static! {
    pub static ref ASSET_TREE_TABLE_NAME: String = format!("{}_{}_assets_tree", VALUE_PROJECT, API_DOMAIN );
}
pub const ASSET_TREE_SON_ID_FIELD_PK: &str = "son_id";
pub const ASSET_TREE_FATHER_ID_FIELD_PK: &str = "father_id";
pub const ASSET_TREE_FATHER_INDEX: &str = "father_index";

lazy_static! {
    pub static ref SHORTER_TABLE_NAME: String = format!("{}_{}_assets_shorter", VALUE_PROJECT, API_DOMAIN );
}
pub const SHORTER_ASSET_ID_FIELD: &str = "asset_id";
pub const SHORTER_FIELD_PK: &str = "shorter";
pub const SHORTER_ASSET_INEX: &str = "shorter_index";

pub struct AssetSchema;
#[async_trait]
impl Schema for AssetSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());
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
            .table_name(ASSETS_TABLE_NAME.clone())
            .key_schema(ks)
            .global_secondary_indexes(second_index)
            .attribute_definitions(asset_ad)
            .attribute_definitions(url_ad)
            .billing_mode(BillingMode::PayPerRequest)
            .stream_specification(
                StreamSpecificationBuilder::default()
                    .stream_enabled(true)
                    .stream_view_type(StreamViewType::NewAndOldImages)
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_ENVIRONMENT.to_string()))
                    .set_value(Some(config.env_vars().environment().unwrap()))
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_PROJECT.to_string()))
                    .set_value(Some(VALUE_PROJECT.to_string()))
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_SERVICE.to_string()))
                    .set_value(Some(API_DOMAIN.to_string()))
                    .build(),
            )
            .send()
            .await?;
        Ok(())
    }
    async fn delete_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());
        client
            .delete_table()
            .table_name(ASSETS_TABLE_NAME.clone())
            .send()
            .await?;

        Ok(())
    }
}

pub struct AssetTreeSchema;
#[async_trait]
impl Schema for AssetTreeSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());
        //async fn create_schema_assets_tree(client: &aws_sdk_dynamodb::Client) -> Result<(), Error> {
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
            .table_name(ASSET_TREE_TABLE_NAME.clone())
            .key_schema(ks1)
            .global_secondary_indexes(second_index)
            .attribute_definitions(ad1)
            .attribute_definitions(ad2)
            .billing_mode(BillingMode::PayPerRequest)
            .stream_specification(
                StreamSpecificationBuilder::default()
                    .stream_enabled(true)
                    .stream_view_type(StreamViewType::NewAndOldImages)
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_ENVIRONMENT.to_string()))
                    .set_value(Some(config.env_vars().environment().unwrap()))
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_PROJECT.to_string()))
                    .set_value(Some(VALUE_PROJECT.to_string()))
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_SERVICE.to_string()))
                    .set_value(Some(API_DOMAIN.to_string()))
                    .build(),
            )
            .send()
            .await?;

        Ok(())
    }
    async fn delete_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());
        client
            .delete_table()
            .table_name(ASSET_TREE_TABLE_NAME.clone())
            .send()
            .await?;

        Ok(())
    }
}
pub struct ShorterSchema;
#[async_trait]
impl Schema for ShorterSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());
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
            .table_name(SHORTER_TABLE_NAME.clone())
            .key_schema(ks)
            .global_secondary_indexes(second_index)
            .attribute_definitions(asset_ad)
            .attribute_definitions(shorter_ad)
            .billing_mode(BillingMode::PayPerRequest)
            .stream_specification(
                StreamSpecificationBuilder::default()
                    .stream_enabled(true)
                    .stream_view_type(StreamViewType::NewAndOldImages)
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_ENVIRONMENT.to_string()))
                    .set_value(Some(config.env_vars().environment().unwrap()))
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_PROJECT.to_string()))
                    .set_value(Some(VALUE_PROJECT.to_string()))
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(TAG_SERVICE.to_string()))
                    .set_value(Some(API_DOMAIN.to_string()))
                    .build(),
            )
            .deletion_protection_enabled(if config.env_vars().environment().unwrap() == PROD_ENV {
                true
            } else {
                false
            })
            .send()
            .await?;
        Ok(())
    }
    async fn delete_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());
        client
            .delete_table()
            .table_name(SHORTER_TABLE_NAME.clone())
            .send()
            .await?;

        Ok(())
    }
}

pub struct AssetAllSchema;
#[async_trait]
impl Schema for AssetAllSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
        AssetSchema::create_schema(config).await?;
        AssetTreeSchema::create_schema(config).await?;
        ShorterSchema::create_schema(config).await?;
        Ok(())
    }

    async fn delete_schema(config: &Config) -> ResultE<()> {
        AssetSchema::delete_schema(config).await?;
        AssetTreeSchema::delete_schema(config).await?;
        ShorterSchema::delete_schema(config).await?;
        Ok(())
    }
}
