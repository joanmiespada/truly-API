use async_trait::async_trait;
use aws_sdk_qldb::types::PermissionsMode;
use lib_config::{
    config::Config,
    environment::{
        ENV_VAR_ENVIRONMENT, ENV_VAR_PROJECT, ENV_VAR_PROJECT_LABEL, ENV_VAR_SERVICE_LABEL,
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

pub struct LedgerSchema;

#[async_trait]
impl Schema for LedgerSchema {
    async fn create_schema(config: &Config) -> ResultE<()> {
         
        let client = aws_sdk_qldb::Client::new(config.aws_config());

        let op = client
            .create_ledger()
            .name(LEDGER_NAME)
            .permissions_mode(PermissionsMode::Standard)
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
                println!("{:?}",e);
        }

        let client = QldbClient::default(LEDGER_NAME, 200).await?;

        let aux2 = client.transaction_within(|client| async move {
            client 
                .query( format!("CREATE TABLE {}", LEDGER_TABLE_NAME).as_str() )
                .execute()
                .await?;
            Ok(())
        })
        .await;
        if let Err(e) = aux2 {
                println!("{:?}",e);
        }

        let aux3 = client.transaction_within(|client| async move {
            client 
                .query( format!("CREATE INDEX ON {} ({})", LEDGER_TABLE_NAME, LEDGER_FIELD_HASH).as_str() )
                .execute()
                .await?;
            Ok(())
        })
        .await;
        if let Err(e) = aux3 {
                println!("{:?}",e);
        }

        let aux4 = client.transaction_within(|client| async move {
            client 
                .query( format!("CREATE INDEX ON {} ({})", LEDGER_TABLE_NAME, LEDGER_FIELD_ASSET_ID).as_str() )
                .execute()
                .await?;
            Ok(())
        })
        .await;
        if let Err(e) = aux4 {
                println!("{:?}",e);
        }

        Ok(())
    }

    async fn delete_schema(_config: &Config) -> ResultE<()> {
        panic!("Delete manually");
        
    }
}

