use crate::SERVICE;
use async_trait::async_trait;
use aws_sdk_dynamodb::types::{
    AttributeDefinition, BillingMode, GlobalSecondaryIndex, KeySchemaElement, KeyType, Projection,
    ProjectionType, ScalarAttributeType, Tag,
};
use lib_config::{
    config::Config,
    environment::{
        ENV_VAR_ENVIRONMENT, ENV_VAR_PROJECT, ENV_VAR_PROJECT_LABEL, ENV_VAR_SERVICE_LABEL,
    },
    result::ResultE,
    schema::Schema,
};

pub const OWNERS_TABLE_NAME: &str = "truly_owners";
pub const OWNER_USER_ID_FIELD_PK: &str = "userId";
pub const OWNER_ASSET_ID_FIELD_PK: &str = "assetId";
pub const OWNERS_USER_ID_INDEX: &str = "user_id_index";
pub const OWNERS_ASSET_ID_INDEX: &str = "asset_id_index";

pub struct OwnerSchema;
#[async_trait]
impl Schema for OwnerSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
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
            .table_name(OWNERS_TABLE_NAME)
            .key_schema(ks1)
            .key_schema(ks2)
            .global_secondary_indexes(second_index_by_user)
            .global_secondary_indexes(second_index_by_asset)
            .attribute_definitions(ad1)
            .attribute_definitions(ad2)
            .billing_mode(BillingMode::PayPerRequest)
            .tags(
                Tag::builder()
                    .set_key(Some(ENV_VAR_ENVIRONMENT.to_string()))
                    .set_value(Some(config.env_vars().environment().clone()))
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(ENV_VAR_PROJECT_LABEL.to_string()))
                    .set_value(Some(ENV_VAR_PROJECT.to_string()))
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(ENV_VAR_SERVICE_LABEL.to_string()))
                    .set_value(Some(SERVICE.to_string()))
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
            .table_name(OWNERS_TABLE_NAME)
            .send()
            .await?;

        Ok(())
    }
}
