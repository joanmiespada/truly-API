use aws_sdk_dynamodb::types::error::ResourceNotFoundException;
use lib_blockchain::repositories::{
    schema_block_tx, schema_blockchain, schema_contract, schema_keypairs,
};
use lib_config::config::Config;
use lib_licenses::repositories::{schema_asset, schema_licenses, schema_owners};
use lib_users::repositories::schema_user;

pub async fn create_schemas(
    table_name: String,
    create: bool,
    delete: bool,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let er = ResourceNotFoundException::builder().build();
    let client = aws_sdk_dynamodb::Client::new(config.aws_config());
    match table_name.as_str() {
        "owners" => {
            if create {
                schema_owners::create_schema_owners(&client).await?
            } else if delete {
                schema_owners::delete_schema_owners(&client).await?
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        "assets" => {
            if create {
                schema_asset::create_schema_assets_all(&client).await?;
            } else if delete {
                schema_asset::delete_schema_assets_all(&client).await?;
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        "keypairs" => {
            if create {
                schema_keypairs::create_schema_keypairs(&client).await?
            } else if delete {
                schema_keypairs::delete_schema_keypairs(&client).await?
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        "transactions" => {
            if create {
                schema_block_tx::create_schema_transactions(&client).await?
            } else if delete {
                schema_block_tx::delete_schema_transactions(&client).await?
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        "users" => {
            if create {
                schema_user::create_schema_users(&client).await?
            } else if delete {
                schema_user::delete_schema_users(&client).await?
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        "blockchains" => {
            if create {
                schema_blockchain::create_schema_blockchains(&client).await?;
            } else if delete {
                schema_blockchain::delete_schema_blockchains(&client).await?;
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        "contracts" => {
            if create {
                schema_contract::create_schema_contracts(&client).await?;
            } else if delete {
                schema_contract::delete_schema_contracts(&client).await?;
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        "licenses" => {
            if create {
                schema_licenses::create_schema_licenses(&client).await?;
            } else if delete {
                schema_licenses::delete_schema_licenses(&client).await?;
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        "all" => {
            if create {
                schema_blockchain::create_schema_blockchains(&client).await?;
                schema_contract::create_schema_contracts(&client).await?;
                schema_owners::create_schema_owners(&client).await?;
                schema_asset::create_schema_assets_all(&client).await?;
                schema_keypairs::create_schema_keypairs(&client).await?;
                schema_block_tx::create_schema_transactions(&client).await?;
                schema_user::create_schema_users(&client).await?;
                schema_licenses::create_schema_licenses(&client).await?;
            } else if delete {
                schema_blockchain::delete_schema_blockchains(&client).await?;
                schema_contract::delete_schema_contracts(&client).await?;
                schema_owners::delete_schema_owners(&client).await?;
                schema_asset::delete_schema_assets_all(&client).await?;
                schema_keypairs::delete_schema_keypairs(&client).await?;
                schema_block_tx::delete_schema_transactions(&client).await?;
                schema_user::delete_schema_users(&client).await?;
                schema_licenses::delete_schema_licenses(&client).await?;
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        _ => {
            return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
        }
    }

    Ok(())
}
