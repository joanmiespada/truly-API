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
    pub static ref LICENSES_TABLE_NAME: String = format!("{}_{}_licenses", VALUE_PROJECT, API_DOMAIN ); 
}

pub const LICENSE_ID_FIELD_PK: &str = "licenseId";
pub const LICENSE_ASSET_ID_FIELD_PK: &str = "assetId";
pub const LICENSES_ASSET_ID_INDEX: &str = "asset_id_index";
pub const LICENSES_LICENSE_ID_INDEX: &str = "license_id_index";

pub struct LicenseSchema;
#[async_trait]
impl Schema for LicenseSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());

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
            .table_name(LICENSES_TABLE_NAME.clone())
            .key_schema(ks1)
            .key_schema(ks2)
            .global_secondary_indexes(second_index_by_asset)
            .global_secondary_indexes(third_index_by_asset)
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

        Ok(())
    }

    async fn delete_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());
        client
            .delete_table()
            .table_name(LICENSES_TABLE_NAME.clone())
            .send()
            .await?;

        Ok(())
    }
}
