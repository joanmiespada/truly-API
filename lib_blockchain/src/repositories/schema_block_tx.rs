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

use crate::SERVICE;

pub const TX_TABLE_NAME: &str = "truly_blockchain_txs";
pub const TX_ASSET_ID_FIELD_PK: &str = "assetId";
pub const TX_TIMESTAMP_PK: &str = "timestamp";
pub const TX_FIELD: &str = "tx";
pub const TX_INDEX_NAME: &str = "tx_index";
pub struct BlockTxSchema;

#[async_trait]
impl Schema for BlockTxSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
        
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());

        let asset_ad = AttributeDefinition::builder()
            .attribute_name(TX_ASSET_ID_FIELD_PK)
            .attribute_type(ScalarAttributeType::S)
            .build();
        let time_ad = AttributeDefinition::builder()
            .attribute_name(TX_TIMESTAMP_PK)
            .attribute_type(ScalarAttributeType::S)
            .build();
        let tx_ad = AttributeDefinition::builder()
            .attribute_name(TX_FIELD)
            .attribute_type(ScalarAttributeType::S)
            .build();

        let ks1 = KeySchemaElement::builder()
            .attribute_name(TX_ASSET_ID_FIELD_PK)
            .key_type(KeyType::Hash)
            .build();
        let ks2 = KeySchemaElement::builder()
            .attribute_name(TX_TIMESTAMP_PK)
            .key_type(KeyType::Range)
            .build();

        let second_index = GlobalSecondaryIndex::builder()
            .index_name(TX_INDEX_NAME)
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name(TX_FIELD)
                    .key_type(KeyType::Hash)
                    .build(),
            )
            .projection(
                Projection::builder()
                    .projection_type(ProjectionType::KeysOnly)
                    .build(),
            )
            .build();

        let op = client
            .create_table()
            .table_name(TX_TABLE_NAME)
            .key_schema(ks1)
            .key_schema(ks2)
            .global_secondary_indexes(second_index)
            .attribute_definitions(asset_ad)
            .attribute_definitions(time_ad)
            .attribute_definitions(tx_ad)
            .billing_mode(BillingMode::PayPerRequest)
            .tags(
                Tag::builder()
                    .set_key(Some(ENV_VAR_ENVIRONMENT.to_string()))
                    .set_value(Some(config.env_vars().environment().unwrap()))
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
            .await;
        match op {
            Err(e) => return Err(e.into()),
            Ok(_) => Ok(()),
        }
    }
    async fn delete_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());
        client
            .delete_table()
            .table_name(TX_TABLE_NAME)
            .send()
            .await?;

        Ok(())
    }
}
