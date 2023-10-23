use async_trait::async_trait;
use aws_sdk_dynamodb::types::{
    builders::StreamSpecificationBuilder, AttributeDefinition, BillingMode, GlobalSecondaryIndex,
    KeySchemaElement, KeyType, Projection, ProjectionType, ScalarAttributeType, StreamViewType,
    Tag,
};
use lib_config::{
    config::Config,
    constants::{API_DOMAIN, TAG_ENVIRONMENT, TAG_PROJECT, TAG_SERVICE, VALUE_PROJECT},
    environment::PROD_ENV,
    result::ResultE,
    schema::{schema_exists, wait_until_schema_is_active, Schema},
};
lazy_static! {
    pub static ref SUBSCRIPTION_TABLE_NAME: String =
        format!("{}_{}_subscription", VALUE_PROJECT, API_DOMAIN);
}

pub const USER_ASSET_INDEX_ID: &str = "UserAssetSubscriptionIndex";
pub const ASSET_USER_INDEX_ID: &str = "AssetUserSubscriptionIndex";
pub const SUBSCRIPTION_ID_FIELD_PK: &str = "subscription_id";
pub const ASSET_ID_FIELD: &str = "asset_id";
pub const USER_ID_FIELD: &str = "user_id";

pub struct SubscriptionSchema;

#[async_trait]
impl Schema for SubscriptionSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
        let exist = schema_exists(config, SUBSCRIPTION_TABLE_NAME.as_str()).await?;
        if exist {
            return Ok(());
        }

        let client = aws_sdk_dynamodb::Client::new(config.aws_config());

        let subscription_pk_attr = AttributeDefinition::builder()
            .attribute_name(SUBSCRIPTION_ID_FIELD_PK)
            .attribute_type(ScalarAttributeType::S)
            .build();

        let user_id_attr = AttributeDefinition::builder()
            .attribute_name(USER_ID_FIELD)
            .attribute_type(ScalarAttributeType::S)
            .build();

        let asset_id_attr = AttributeDefinition::builder()
            .attribute_name( ASSET_ID_FIELD)
            .attribute_type(ScalarAttributeType::S)
            .build();

        // let subscription_id_attr = AttributeDefinition::builder()
        //     .attribute_name(SUBSCRIPTION_ID_FIELD)
        //     .attribute_type(ScalarAttributeType::S)
        //     .build();

        let key_schema = KeySchemaElement::builder()
            .attribute_name(SUBSCRIPTION_ID_FIELD_PK)
            .key_type(KeyType::Hash)
            .build();

        // let sort_key = KeySchemaElement::builder()
        //     .attribute_name(SUBSCRIPTION_ID_FIELD)
        //     .key_type(KeyType::Range)
        //     .build();

        let gsi1 = GlobalSecondaryIndex::builder()
            .index_name(USER_ASSET_INDEX_ID)
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name(USER_ID_FIELD)
                    .key_type(KeyType::Hash)
                    .build(),
            )
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name(ASSET_ID_FIELD )
                    .key_type(KeyType::Range)
                    .build(),
            )
            .projection(
                Projection::builder()
                    .projection_type(ProjectionType::All)
                    .build(),
            )
            .build();

        let gsi2 = GlobalSecondaryIndex::builder()
            .index_name(ASSET_USER_INDEX_ID)
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name(ASSET_ID_FIELD)
                    .key_type(KeyType::Hash)
                    .build(),
            )
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name(USER_ID_FIELD )
                    .key_type(KeyType::Range)
                    .build(),
            )
            .projection(
                Projection::builder()
                    .projection_type(ProjectionType::All)
                    .build(),
            )
            .build();
        // let gsi2 = GlobalSecondaryIndex::builder()
        //     .index_name(SUBSCRIPTION_INDEX_ID)
        //     .key_schema(
        //         KeySchemaElement::builder()
        //             .attribute_name(SUBSCRIPTION_ID_FIELD)
        //             .key_type(KeyType::Hash)
        //             .build(),
        //     )
        //     .projection(
        //         Projection::builder()
        //             .projection_type(ProjectionType::All)
        //             .build(),
        //     )
        //     .build();
        

        client
            .create_table()
            .table_name(SUBSCRIPTION_TABLE_NAME.clone())
            .attribute_definitions(user_id_attr)
            .attribute_definitions(asset_id_attr)
            //.attribute_definitions(subscription_id_attr)
            .attribute_definitions(subscription_pk_attr )
            .key_schema(key_schema)
            //.key_schema(sort_key)
            .global_secondary_indexes(gsi1)
            .global_secondary_indexes(gsi2)
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

        wait_until_schema_is_active(config, SUBSCRIPTION_TABLE_NAME.as_str()).await?;

        Ok(())
    }

    async fn delete_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());
        client
            .delete_table()
            .table_name(SUBSCRIPTION_TABLE_NAME.clone())
            .send()
            .await?;

        Ok(())
    }
}
