use async_trait::async_trait;

use aws_sdk_qldb::types::PermissionsMode;
use aws_sdk_qldbsession::{
    operation::send_command::{builders::SendCommandFluentBuilder, SendCommandInput},
    types::{ExecuteStatementRequest, StartSessionRequest},
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

pub const LEDGER_NAME: &str = "truly_assets_ledger";
pub const LEDGER_TABLE_NAME: &str = "Asset";
pub const LEDGER_FIELD_ASSET_ID: &str = "asset_id";
pub const LEDGER_FIELD_Y: &str = "y";

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
            .tags(ENV_VAR_SERVICE_LABEL.to_string(), Some(SERVICE.to_string()))
            .send()
            .await;
        match op {
            Err(e) => return Err(e.into()),
            Ok(_) => Ok(()),
        }
    }
    async fn delete_schema(config: &Config) -> ResultE<()> {
        let client = aws_sdk_qldb::Client::new(config.aws_config());
        client
            .delete_ledger()
            .name(LEDGER_TABLE_NAME)
            .send()
            .await?;

        Ok(())
    }
}

impl LedgerSchema {
    pub async fn create_table(config: &Config) -> ResultE<()> {
        let client = aws_sdk_qldbsession::Client::new(config.aws_config());

        /*
        let conf =aws_sdk_qldbsession::Config::new(config.aws_config());

        let mut op = SendCommandInput::builder().start_session(
            StartSessionRequest::builder().ledger_name(LEDGER_NAME).build()
        ).build()
        .unwrap()
        .make_operation(&conf)
        .await?;

        op.properties_mut().insert(val)*/

        let op = client
            .send_command()
            .start_session(
                StartSessionRequest::builder()
                    .ledger_name(LEDGER_NAME)
                    .build(),
            )
            .execute_statement(
                ExecuteStatementRequest::builder()
                    .statement(format!("CREATE TABLE {}", LEDGER_TABLE_NAME))
                    .build(),
            )
            .execute_statement(
                ExecuteStatementRequest::builder()
                    .statement(format!(
                        "CREATE INDEX ON {}({})",
                        LEDGER_TABLE_NAME, LEDGER_FIELD_ASSET_ID
                    ))
                    .build(),
            )
            .send()
            .await;

        match op {
            Err(e) => return Err(e.into()),
            Ok(_) => Ok(()),
        }
    }
}
