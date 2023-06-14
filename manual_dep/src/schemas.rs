use aws_sdk_dynamodb::types::error::ResourceNotFoundException;
use lib_blockchain::repositories::{
    schema_block_tx::BlockTxSchema, schema_contract::ContractSchema, schema_keypairs::KeyPairSchema,
};
use lib_config::config::Config;
use lib_licenses::repositories::{schema_asset::AssetAllSchema, schema_licenses::LicenseSchema, schema_owners::OwnerSchema};
use lib_users::repositories::schema_user::UserAllSchema;
use lib_blockchain::repositories::schema_blockchain::BlockchainSchema;
use lib_config::schema::Schema;

pub async fn create_schemas(
    table_name: String,
    create: bool,
    delete: bool,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let er = ResourceNotFoundException::builder().build();
    match table_name.as_str() {
        "owners" => {
            if create {
                OwnerSchema::create_schema(config).await?;
            } else if delete {
                OwnerSchema::delete_schema(config).await?;
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        "assets" => {
            if create {
                AssetAllSchema::create_schema(config).await?
            } else if delete {
                AssetAllSchema::delete_schema(config).await?
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        "keypairs" => {
            if create {
                KeyPairSchema::create_schema(&config).await?;
            } else if delete {
                KeyPairSchema::delete_schema(&config).await?;
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        "transactions" => {
            if create {
                BlockTxSchema::create_schema(&config).await?;
            } else if delete {
                BlockTxSchema::delete_schema(&config).await?;
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        "users" => {
            if create {
                UserAllSchema::create_schema(config).await?;
                //schema_user::create_schema_users(&client).await?
            } else if delete {
                UserAllSchema::delete_schema(config).await?;
                //schema_user::delete_schema_users(&client).await?
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        "blockchains" => {
            if create {
                BlockchainSchema::create_schema(config).await?;
            } else if delete {
                BlockchainSchema::delete_schema(config).await?;
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        "contracts" => {
            if create {
                ContractSchema::create_schema(config).await?;
            } else if delete {
                ContractSchema::delete_schema(config).await?;
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        "licenses" => {
            if create {
                LicenseSchema::create_schema(config).await?;
                //schema_licenses::create_schema_licenses(&client).await?;
            } else if delete {
                LicenseSchema::delete_schema(config).await?;
                //schema_licenses::delete_schema_licenses(&client).await?;
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        "all" => {
            if create {
                BlockchainSchema::create_schema(config).await?;
                //schema_contract::create_schema_contracts(&client).await?;
                ContractSchema::create_schema(config).await?;
                OwnerSchema::create_schema(config).await?;
                //schema_owners::create_schema_owners(&client).await?;
                AssetAllSchema::create_schema(config).await?;
                //schema_asset::create_schema_assets_all(&client).await?;
                KeyPairSchema::create_schema(&config).await?;
                //schema_keypairs::create_schema_keypairs(&client).await?;
                //schema_block_tx::create_schema_transactions(&client).await?;
                BlockTxSchema::create_schema(&config).await?;
                //schema_user::create_schema_users(&client).await?;
                UserAllSchema::create_schema(config).await?;
                //schema_licenses::create_schema_licenses(&client).await?;
                LicenseSchema::create_schema(config).await?;
            } else if delete {
                BlockchainSchema::delete_schema(config).await?;
                //schema_contract::delete_schema_contracts(&client).await?;
                ContractSchema::delete_schema(config).await?;
                OwnerSchema::delete_schema(config).await?; 
                // schema_owners::delete_schema_owners(&client).await?;
                AssetAllSchema::delete_schema(config).await?;
                 //schema_asset::delete_schema_assets_all(&client).await?;
                //schema_keypairs::delete_schema_keypairs(&client).await?;
                KeyPairSchema::delete_schema(&config).await?;
                //schema_block_tx::delete_schema_transactions(&client).await?;
                BlockTxSchema::delete_schema(&config).await?;
                UserAllSchema::delete_schema(config).await?;
                //schema_user::delete_schema_users(&client).await?;
                //schema_licenses::delete_schema_licenses(&client).await?;
                LicenseSchema::delete_schema(config).await?;
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
