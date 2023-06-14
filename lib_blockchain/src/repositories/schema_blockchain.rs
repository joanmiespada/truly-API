use aws_sdk_dynamodb::{
    types::{
        AttributeDefinition, BillingMode, KeySchemaElement, KeyType, ScalarAttributeType, Tag,
    },
};
use lib_config::{
    config::Config, environment::{ENV_VAR_ENVIRONMENT, ENV_VAR_PROJECT_LABEL, ENV_VAR_PROJECT, ENV_VAR_SERVICE_LABEL}, result::ResultE, schema::Schema,
};
use async_trait::async_trait;
use crate::SERVICE;

pub const BLOCKCHAIN_TABLE_NAME: &str = "truly_blockchain";
pub const BLOCKCHAIN_ID_FIELD_PK: &str = "blockchain_id";

pub struct BlockchainSchema;

#[async_trait]
impl Schema for BlockchainSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());
        let id_ad = AttributeDefinition::builder()
            .attribute_name(BLOCKCHAIN_ID_FIELD_PK)
            .attribute_type(ScalarAttributeType::S)
            .build();

        let ks1 = KeySchemaElement::builder()
            .attribute_name(BLOCKCHAIN_ID_FIELD_PK)
            .key_type(KeyType::Hash)
            .build();

        let op = client
            .create_table()
            .table_name(BLOCKCHAIN_TABLE_NAME)
            .key_schema(ks1)
            .attribute_definitions(id_ad)
            .billing_mode(BillingMode::PayPerRequest)
            .tags(
                Tag::builder()
                    .set_key(Some(ENV_VAR_ENVIRONMENT.to_string()))
                    .set_value(Some( config.env_vars().environment().clone() ))
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(ENV_VAR_PROJECT_LABEL.to_string()))
                    .set_value(Some( ENV_VAR_PROJECT.to_string() ))
                    .build(),
            )
            .tags(
                Tag::builder()
                    .set_key(Some(ENV_VAR_SERVICE_LABEL.to_string()))
                    .set_value(Some( SERVICE.to_string() ))
                    .build(),
            );
        let op = op.send().await;
        match op {
            Err(e) => return Err(e.into()),
            Ok(_) => Ok(()),
        }
    }
    async fn delete_schema(config: &Config) -> ResultE<()> {

        let client = aws_sdk_dynamodb::Client::new(config.aws_config());
        client
            .delete_table()
            .table_name(BLOCKCHAIN_TABLE_NAME)
            .send()
            .await?;

        Ok(())
    }
}
