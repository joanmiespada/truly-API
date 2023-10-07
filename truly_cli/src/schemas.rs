use aws_sdk_dynamodb::types::error::ResourceNotFoundException;
//use lib_blockchain::repositories::schema_blockchain::BlockchainSchema;
//use lib_blockchain::repositories::{
//    schema_block_tx::BlockTxSchema, schema_contract::ContractSchema, schema_keypairs::KeyPairSchema,
//};
use lib_config::config::Config;
use lib_config::schema::Schema;
use lib_ledger::repository::schema_ledger::LedgerSchema;
use lib_licenses::repositories::{
    schema_asset::{AssetAllSchema, ASSETS_TABLE_NAME}, schema_licenses::{LicenseSchema, LICENSES_TABLE_NAME}, schema_owners::{OwnerSchema, OWNERS_TABLE_NAME}, 
};
use lib_users::repositories::schema_user::{UserAllSchema, USERS_TABLE_NAME };

pub async fn create_schemas(
    table_name: String,
    create: bool,
    delete: bool,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let er = ResourceNotFoundException::builder().build();
    match table_name.as_str() {
        OWNERS_TABLE_NAME => {
            if create {
                OwnerSchema::create_schema(config).await?;
            } else if delete {
                OwnerSchema::delete_schema(config).await?;
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        ASSETS_TABLE_NAME => {
            if create {
                AssetAllSchema::create_schema(config).await?
            } else if delete {
                AssetAllSchema::delete_schema(config).await?
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        // "ledger" => {
        //     if create {
        //         LedgerSchema::create_schema(config).await?
        //     } else if delete {
        //         LedgerSchema::delete_schema(config).await?
        //     } else {
        //         return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
        //     }
        // }
        // "keypairs" => {
        //     if create {
        //         KeyPairSchema::create_schema(&config).await?;
        //     } else if delete {
        //         KeyPairSchema::delete_schema(&config).await?;
        //     } else {
        //         return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
        //     }
        // }
        // "transactions" => {
        //     if create {
        //         BlockTxSchema::create_schema(&config).await?;
        //     } else if delete {
        //         BlockTxSchema::delete_schema(&config).await?;
        //     } else {
        //         return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
        //     }
        // }
        USERS_TABLE_NAME => {
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
        // "blockchains" => {
        //     if create {
        //         BlockchainSchema::create_schema(config).await?;
        //     } else if delete {
        //         BlockchainSchema::delete_schema(config).await?;
        //     } else {
        //         return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
        //     }
        // }
        // "contracts" => {
        //     if create {
        //         ContractSchema::create_schema(config).await?;
        //     } else if delete {
        //         ContractSchema::delete_schema(config).await?;
        //     } else {
        //         return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
        //     }
        // }
        LICENSES_TABLE_NAME => {
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
                //BlockchainSchema::create_schema(config).await?;
                //ContractSchema::create_schema(config).await?;
                OwnerSchema::create_schema(config).await?;
                AssetAllSchema::create_schema(config).await?;
                //KeyPairSchema::create_schema(&config).await?;
                //BlockTxSchema::create_schema(&config).await?;
                UserAllSchema::create_schema(config).await?;
                LicenseSchema::create_schema(config).await?;
                LedgerSchema::create_schema(config).await?
            } else if delete {
                //BlockchainSchema::delete_schema(config).await?;
                //ContractSchema::delete_schema(config).await?;
                OwnerSchema::delete_schema(config).await?;
                AssetAllSchema::delete_schema(config).await?;
                //KeyPairSchema::delete_schema(&config).await?;
                //BlockTxSchema::delete_schema(&config).await?;
                UserAllSchema::delete_schema(config).await?;
                LicenseSchema::delete_schema(config).await?;
                LedgerSchema::delete_schema(config).await?
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
