use aws_sdk_dynamodb::types::error::ResourceNotFoundException;
use lib_config::config::Config;
use lib_config::schema::Schema;
use lib_licenses::repositories::{
    schema_asset::AssetAllSchema, schema_licenses::LicenseSchema, schema_owners::OwnerSchema,
};
use lib_engage::repositories::schema_subscription::SubscriptionSchema;
use lib_licenses::{
    services::assets::SERVICE as ASSET_SERVICE, services::licenses::SERVICE as LICENSE_SERVICE,
    services::owners::SERVICE as OWNER_SERVICE,
};
use lib_engage::services::subscription::SERVICE as SUBSCRIPTION_SERVICE;
use lib_users::repositories::schema_user::UserAllSchema;
use lib_users::SERVICE as USER_SERVICE;

pub async fn create_schemas(
    service_name: String,
    create: bool,
    delete: bool,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let er = ResourceNotFoundException::builder()
        .message("Not found")
        .build();

    match service_name.as_str() {
        OWNER_SERVICE => {
            if create {
                OwnerSchema::create_schema(config).await?;
            } else if delete {
                OwnerSchema::delete_schema(config).await?;
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        ASSET_SERVICE => {
            if create {
                AssetAllSchema::create_schema(config).await?
            } else if delete {
                AssetAllSchema::delete_schema(config).await?
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }

        USER_SERVICE => {
            if create {
                let aux = UserAllSchema::create_schema(config).await;
                if let Err(err) = aux {
                    println!("Error: {}", err)
                }
                //schema_user::create_schema_users(&client).await?
            } else if delete {
                UserAllSchema::delete_schema(config).await?;
                //schema_user::delete_schema_users(&client).await?
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }

        LICENSE_SERVICE => {
            if create {
                LicenseSchema::create_schema(config).await?;
            } else if delete {
                LicenseSchema::delete_schema(config).await?;
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        SUBSCRIPTION_SERVICE => {
            if create {
                SubscriptionSchema::create_schema(config).await?;
            } else if delete {
                SubscriptionSchema::delete_schema(config).await?;
            } else {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        }
        "all" => {
            if create {
                OwnerSchema::create_schema(config).await?;
                AssetAllSchema::create_schema(config).await?;
                UserAllSchema::create_schema(config).await?;
                LicenseSchema::create_schema(config).await?;
                SubscriptionSchema::create_schema(config).await?;
            } else if delete {
                OwnerSchema::delete_schema(config).await?;
                AssetAllSchema::delete_schema(config).await?;
                UserAllSchema::delete_schema(config).await?;
                LicenseSchema::delete_schema(config).await?;
                SubscriptionSchema::delete_schema(config).await?;
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
