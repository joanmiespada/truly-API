use async_trait::async_trait;
use aws_sdk_dynamodb::types::{
    builders::StreamSpecificationBuilder, AttributeDefinition, BillingMode,
    KeySchemaElement, KeyType, ScalarAttributeType, StreamViewType,
    Tag, LocalSecondaryIndex, Projection, ProjectionType,
};
use lib_config::{
    config::Config,
    constants::{API_DOMAIN, TAG_ENVIRONMENT, TAG_PROJECT, TAG_SERVICE, VALUE_PROJECT},
    environment::PROD_ENV,
    result::ResultE,
    schema::{schema_exists, wait_until_schema_is_active, Schema},
};
lazy_static! {
    pub static ref ALERT_SIMILARS_TABLE_NAME: String =
        format!("{}_{}_alert_similars", VALUE_PROJECT, API_DOMAIN);
}

pub const ALERT_SIMILAR_ID_FIELD_PK: &str = "alert_id";
pub const CREATION_TIME: &str = "creation_time";
pub const TIME_INDEX_NAME: &str = "time_index";

pub struct AlertSimilarSchema;

#[async_trait]
impl Schema for AlertSimilarSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
        let exist = schema_exists(config, ALERT_SIMILARS_TABLE_NAME.as_str()).await?;
        if exist {
            return Ok(());
        }

        let client = aws_sdk_dynamodb::Client::new(config.aws_config());

        let notification_pk_attr = AttributeDefinition::builder()
            .attribute_name(ALERT_SIMILAR_ID_FIELD_PK)
            .attribute_type(ScalarAttributeType::S)
            .build();

        let creation_time_attr = AttributeDefinition::builder()
            .attribute_name(CREATION_TIME)
            .attribute_type(ScalarAttributeType::S)
            .build();

        let key_schema = KeySchemaElement::builder()
            .attribute_name(ALERT_SIMILAR_ID_FIELD_PK)
            .key_type(KeyType::Hash)
            .build();

        let second_index = LocalSecondaryIndex::builder()
            .index_name(TIME_INDEX_NAME)
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name(CREATION_TIME)
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
            .table_name(ALERT_SIMILARS_TABLE_NAME.clone())
            .attribute_definitions(notification_pk_attr )
            .attribute_definitions(creation_time_attr )
            .key_schema(key_schema)
            .local_secondary_indexes(second_index)
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

        wait_until_schema_is_active(config, ALERT_SIMILARS_TABLE_NAME.as_str()).await?;

        Ok(())
    }

    async fn delete_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());
        client
            .delete_table()
            .table_name(ALERT_SIMILARS_TABLE_NAME.clone())
            .send()
            .await?;

        Ok(())
    }
}
