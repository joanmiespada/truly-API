use aws_sdk_dynamodb::types::error::ResourceNotFoundException;
use lib_ledger::repository::schema_ledger::LedgerSchema;

use lib_config::{config::Config, schema::Schema};

pub async fn manage_ledger(
    create: bool,
    delete: bool,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let er = ResourceNotFoundException::builder().build();
    if create {

        LedgerSchema::create_schema(config).await?; 
        // qldb client doesn't work properly
        //LedgerSchema::create_qldb_schema(config).await?;
        
    } else if delete {
        LedgerSchema::delete_schema(config).await?; 
        // qldb client doesn't work properly
        //LedgerSchema::delete_qldb_schema(config).await?;
    } else {
        return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
    }

    Ok(())
}
