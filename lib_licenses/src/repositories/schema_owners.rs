use async_trait::async_trait;
use aws_sdk_dynamodb::types::{
    builders::StreamSpecificationBuilder,
    AttributeDefinition,
    BillingMode,
    GlobalSecondaryIndex,
    KeySchemaElement,
    KeyType,
    Projection,
    ProjectionType,
    ScalarAttributeType,
    StreamViewType,
    Tag,
};
use lib_config::{
    config::Config,
    environment::PROD_ENV,
    result::ResultE,
    schema::{Schema,schema_exists,wait_until_schema_is_active},
    constants::{
        VALUE_PROJECT, API_DOMAIN, TAG_PROJECT, TAG_SERVICE, TAG_ENVIRONMENT
    }
};


lazy_static! {
    pub static ref OWNERS_TABLE_NAME: String = format!("{}_{}_owners", VALUE_PROJECT, API_DOMAIN);
}

pub const OWNER_USER_ID_FIELD_PK: &str = "userId";
pub const OWNER_ASSET_ID_FIELD_PK: &str = "assetId";
pub const OWNERS_USER_ID_INDEX: &str = "user_id_index";
pub const OWNERS_ASSET_ID_INDEX: &str = "asset_id_index";

pub struct OwnerSchema;
#[async_trait]
impl Schema for OwnerSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {

        let exist = schema_exists(config, OWNERS_TABLE_NAME.as_str()).await?;
        if exist{
            return Ok(())
        }


        let client = aws_sdk_dynamodb::Client::new(config.aws_config());
        let ad1 = AttributeDefinition::builder()
            .attribute_name(OWNER_USER_ID_FIELD_PK)
            .attribute_type(ScalarAttributeType::S)
            .build();
        let ad2 = AttributeDefinition::builder()
            .attribute_name(OWNER_ASSET_ID_FIELD_PK)
            .attribute_type(ScalarAttributeType::S)
            .build();

        let ks1 = KeySchemaElement::builder()
            .attribute_name(OWNER_USER_ID_FIELD_PK)
            .key_type(KeyType::Hash)
            .build();
        let ks2 = KeySchemaElement::builder()
            .attribute_name(OWNER_ASSET_ID_FIELD_PK)
            .key_type(KeyType::Range)
            .build();

        let second_index_by_user = GlobalSecondaryIndex::builder()
            .index_name(OWNERS_USER_ID_INDEX)
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name(OWNER_USER_ID_FIELD_PK)
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
                    .attribute_name(OWNER_ASSET_ID_FIELD_PK)
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
            .table_name(OWNERS_TABLE_NAME.clone())
            .key_schema(ks1)
            .key_schema(ks2)
            .global_secondary_indexes(second_index_by_user)
            .global_secondary_indexes(second_index_by_asset)
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
            .deletion_protection_enabled(if config.env_vars().environment().unwrap() == PROD_ENV {
                true
            } else {
                false
            })
            .send()
            .await?;

        wait_until_schema_is_active(config, OWNERS_TABLE_NAME.as_str()).await?;

        Ok(())
    }

    async fn delete_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());
        client
            .delete_table()
            .table_name(OWNERS_TABLE_NAME.clone())
            .send()
            .await?;

        Ok(())
    }
}


