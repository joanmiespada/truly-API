use crate::SERVICE;
use async_trait::async_trait;
use aws_sdk_dynamodb::types::{
    builders::StreamSpecificationBuilder, AttributeDefinition, BillingMode, GlobalSecondaryIndex,
    KeySchemaElement, KeyType, Projection, ProjectionType, ScalarAttributeType, StreamViewType,
    Tag,
};
use lib_config::{
    config::Config,
    environment::{
        ENV_VAR_ENVIRONMENT, ENV_VAR_PROJECT, ENV_VAR_PROJECT_LABEL, ENV_VAR_SERVICE_LABEL,
    },
    result::ResultE,
    schema::Schema,
};

pub const CONTRACT_TABLE_NAME: &str = "truly_contract";
pub const CONTRACT_ID_FIELD_PK: &str = "contract_id";
pub const CONTRACT_BLOCKCHAIN_FIELD: &str = "blockchain";
pub const CONTRACT_BLOCKCHAIN_INDEX: &str = "blockchain_index";
pub const CONTRACT_STATUS_FIELD_NAME: &str = "status";

pub struct ContractSchema;
#[async_trait]
impl Schema for ContractSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());
        //pub async fn create_schema_contracts(client: &aws_sdk_dynamodb::Client) -> Result<(), Error> {
        let id_ad = AttributeDefinition::builder()
            .attribute_name(CONTRACT_ID_FIELD_PK)
            .attribute_type(ScalarAttributeType::N)
            .build();

        let blockchain_ad = AttributeDefinition::builder()
            .attribute_name(CONTRACT_BLOCKCHAIN_FIELD)
            .attribute_type(ScalarAttributeType::S)
            .build();
        let status_ad = AttributeDefinition::builder()
            .attribute_name(CONTRACT_STATUS_FIELD_NAME)
            .attribute_type(ScalarAttributeType::S)
            .build();

        let ks1 = KeySchemaElement::builder()
            .attribute_name(CONTRACT_ID_FIELD_PK)
            .key_type(KeyType::Hash)
            .build();

        let second_index = GlobalSecondaryIndex::builder()
            .index_name(CONTRACT_BLOCKCHAIN_INDEX)
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name(CONTRACT_BLOCKCHAIN_FIELD)
                    .key_type(KeyType::Hash)
                    .build(),
            )
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name(CONTRACT_STATUS_FIELD_NAME)
                    .key_type(KeyType::Range)
                    .build(),
            )
            .projection(
                Projection::builder()
                    .projection_type(ProjectionType::All) //due to is a very short table and very limitted rows
                    .build(),
            )
            .build();

        let op = client
            .create_table()
            .table_name(CONTRACT_TABLE_NAME)
            .key_schema(ks1)
            .global_secondary_indexes(second_index)
            .attribute_definitions(id_ad)
            .attribute_definitions(blockchain_ad)
            .attribute_definitions(status_ad)
            .billing_mode(BillingMode::PayPerRequest)
            .stream_specification(
                StreamSpecificationBuilder::default()
                    .stream_enabled(true)
                    .stream_view_type(StreamViewType::NewAndOldImages)
                    .build(),
            )
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
        //pub async fn delete_schema_contracts(client: &aws_sdk_dynamodb::Client) -> Result<(), Error> {
        client
            .delete_table()
            .table_name(CONTRACT_TABLE_NAME)
            .send()
            .await?;

        Ok(())
    }
}
