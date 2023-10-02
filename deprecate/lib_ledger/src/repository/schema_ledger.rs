use std::{thread, time::Duration};

use async_trait::async_trait;
use aws_sdk_dynamodb::types::{
    builders::StreamSpecificationBuilder, AttributeDefinition, BillingMode, GlobalSecondaryIndex,
    KeySchemaElement, KeyType, Projection, ProjectionType, ScalarAttributeType, StreamViewType,
    Tag,
};
use aws_sdk_qldb::types::{LedgerState, PermissionsMode};
use lib_config::{
    config::Config,
    environment::{
        ENV_VAR_ENVIRONMENT, ENV_VAR_PROJECT, ENV_VAR_PROJECT_LABEL, ENV_VAR_SERVICE_LABEL,
        PROD_ENV,
    },
    result::ResultE,
    schema::Schema,
};

use crate::SERVICE;
use qldb::QldbClient;

pub const LEDGER_NAME: &str = "truly-assets-ledger";
pub const LEDGER_TABLE_NAME: &str = "Asset";
pub const LEDGER_FIELD_ASSET_ID: &str = "asset_id"; //this field name must match with model field
pub const LEDGER_FIELD_HASH: &str = "asset_hash"; //idem
pub const LEDGER_FIELD_HASH_ALGO: &str = "asset_hash_algorithm"; //idem
pub const LEDGER_FIELD_CREATION_TIME: &str = "asset_creation_time"; //idem

pub const DYNAMODB_TABLE_NAME: &str = "truly_ledgers";
pub const DYNAMODB_ASSET_ID_FIELD_PK: &str = "asset_id";
pub const DYNAMODB_TABLE_INDEX_HASH: &str = "hashId";
pub const DYNAMODB_HASH_FIELD_NAME: &str = "asset_hash";

pub struct LedgerSchema;

impl LedgerSchema {
    async fn create_dynamodb_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());
        let asset_ad = AttributeDefinition::builder()
            .attribute_name(DYNAMODB_ASSET_ID_FIELD_PK)
            .attribute_type(ScalarAttributeType::S)
            .build();
        let hash_ad = AttributeDefinition::builder()
            .attribute_name(DYNAMODB_HASH_FIELD_NAME)
            .attribute_type(ScalarAttributeType::S)
            .build();

        let ks = KeySchemaElement::builder()
            .attribute_name(DYNAMODB_ASSET_ID_FIELD_PK)
            .key_type(KeyType::Hash)
            .build();

        let second_index = GlobalSecondaryIndex::builder()
            .index_name(DYNAMODB_TABLE_INDEX_HASH)
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name(DYNAMODB_HASH_FIELD_NAME)
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
            .table_name(DYNAMODB_TABLE_NAME)
            .key_schema(ks)
            .global_secondary_indexes(second_index)
            .attribute_definitions(asset_ad)
            .attribute_definitions(hash_ad)
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
            .deletion_protection_enabled(if config.env_vars().environment().unwrap() == PROD_ENV {
                true
            } else {
                false
            })
            .send()
            .await?;

        Ok(())
    }

    pub async fn create_qldb_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_qldb::Client::new(config.aws_config());

        let op = client
            .create_ledger()
            .name(LEDGER_NAME)
            .permissions_mode(PermissionsMode::Standard)
            .deletion_protection(if config.env_vars().environment().unwrap() == PROD_ENV {
                true
            } else {
                false
            })
            .tags(
                ENV_VAR_ENVIRONMENT.to_string(),
                Some(config.env_vars().environment().unwrap()),
            )
            .tags(
                ENV_VAR_PROJECT_LABEL.to_string(),
                Some(ENV_VAR_PROJECT.to_string()),
            )
            .tags(ENV_VAR_SERVICE_LABEL.to_string(), Some(SERVICE.to_string()));

        let aux = op.send().await;
        if let Err(e) = aux {
            println!("{:?}", e);
        }

        let mut sts;
        loop {
            let ledger_state = client.describe_ledger().name(LEDGER_NAME).send().await?;
            sts = ledger_state.state().unwrap().to_owned();
            if sts != LedgerState::Creating {
                break;
            } else {
                thread::sleep(Duration::from_secs(5));
            }
        }

        if sts != LedgerState::Active {
            panic!("Ledger creation has failed!");
        }

        let client = QldbClient::default(LEDGER_NAME, 200).await?;

        let aux2 = client
            .transaction_within(|client| async move {
                client
                    .query(format!("CREATE TABLE {}", LEDGER_TABLE_NAME).as_str())
                    .execute()
                    .await?;
                Ok(())
            })
            .await;
        if let Err(e) = aux2 {
            println!("{:?}", e);
        }

        let aux3 = client
            .transaction_within(|client| async move {
                client
                    .query(
                        format!(
                            "CREATE INDEX ON {} ({})",
                            LEDGER_TABLE_NAME, LEDGER_FIELD_HASH
                        )
                        .as_str(),
                    )
                    .execute()
                    .await?;
                Ok(())
            })
            .await;
        if let Err(e) = aux3 {
            println!("{:?}", e);
        }

        let aux4 = client
            .transaction_within(|client| async move {
                client
                    .query(
                        format!(
                            "CREATE INDEX ON {} ({})",
                            LEDGER_TABLE_NAME, LEDGER_FIELD_ASSET_ID
                        )
                        .as_str(),
                    )
                    .execute()
                    .await?;
                Ok(())
            })
            .await;
        if let Err(e) = aux4 {
            println!("{:?}", e);
        }

        Ok(())
    }

    pub async fn delete_qldb_schema(config: &Config) -> ResultE<()> {
        if config.env_vars().environment().unwrap() != PROD_ENV {
            let client = aws_sdk_qldb::Client::new(config.aws_config());
            client.delete_ledger().name(LEDGER_NAME).send().await?;
        }else{
            println!("remove ledger in prod is forbidden.")
        } 
        Ok(())  
    }

}

#[async_trait]
impl Schema for LedgerSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
        LedgerSchema::create_dynamodb_schema(config).await?;
        println!("ledger dynamodb table created successfully.");
        println!("Please, run ledger creation by script.");
        Ok(())
    }

    async fn delete_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_dynamodb::Client::new(config.aws_config());
        client
            .delete_table()
            .table_name(DYNAMODB_TABLE_NAME)
            .send()
            .await?;

        println!("ledger dynamodb table deleted successfully.");
        println!("Please, delete ledger manually.");
        

        Ok(())
    }
}
