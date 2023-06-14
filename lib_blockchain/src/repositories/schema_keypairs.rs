use crate::SERVICE;
use async_trait::async_trait;
use aws_sdk_dynamodb::{
    types::{
        AttributeDefinition, BillingMode, GlobalSecondaryIndex, KeySchemaElement, KeyType,
        Projection, ProjectionType, ScalarAttributeType, Tag,
    },
};
use lib_config::{
    config::Config,
    environment::{
        ENV_VAR_ENVIRONMENT, ENV_VAR_PROJECT, ENV_VAR_PROJECT_LABEL, ENV_VAR_SERVICE_LABEL,
    },
    result::ResultE,
    schema::Schema,
};

pub const KEYPAIRS_TABLE_NAME: &str = "truly_users_keypairs";
pub const KEYPAIRS_USER_ID_FIELD_PK: &str = "userId";
pub const KEYPAIRS_ADDRESS_FIELD: &str = "address";
pub const KEYPAIRS_ADDRESS_INDEX_NAME: &str = "address_index";
pub const KEYPAIRS_PUBLIC_FIELD: &str = "public_key_enc";
pub const KEYPAIRS_PRIVATE_FIELD: &str = "private_key_enc";

pub struct KeyPairSchema;
#[async_trait]
impl Schema for KeyPairSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());

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

        client
            .create_table()
            .table_name(KEYPAIRS_TABLE_NAME)
            .key_schema(ks_by_user_id)
            .global_secondary_indexes(second_index)
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
            .table_name(KEYPAIRS_TABLE_NAME)
            .send()
            .await?;

        Ok(())
    }
}
