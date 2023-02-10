use aws_sdk_dynamodb::error::ResourceNotFoundException;
use lib_config::config::Config;
use lib_config::infra::{create_key, create_secret_manager_keys, create_secret_manager_secret_key};
use lib_licenses::repositories::{schema_asset, schema_keypairs, schema_owners};
use lib_users::repositories::schema_user;
use std::process;
use structopt::StructOpt;

#[allow(unused_variables)]
async fn command(
    Opt {
        table,
        create,
        delete,
        environment,
        secrets,
        keys,
    }: Opt,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::new();
    config.setup().await;
    let client = aws_sdk_dynamodb::Client::new(config.aws_config());

    let er = ResourceNotFoundException::builder().build();

    match table {
        None => {}
        Some(table_name) => match table_name.as_str() {
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
                    schema_asset::create_schema_assets(&client).await?
                } else if delete {
                    schema_asset::delete_schema_assets(&client).await?
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
            "users" => {
                if create {
                    schema_user::create_schema_users(&client).await?
                } else if delete {
                    schema_user::delete_schema_users(&client).await?
                } else {
                    return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
                }
            }
            _ => {
                return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
            }
        },
    }
    match secrets {
        None => {}
        Some(scretes_ok) => {
            let client_sec = aws_sdk_secretsmanager::client::Client::new(config.aws_config());
            let secrets_json = r#"
                    {
                        "HMAC_SECRET" : "localtest_hmac_1234RGsdfg#$%",
                        "JWT_TOKEN_BASE": "localtest_jwt_sd543ERGds235$%^"
                    }
                    "#;

            create_secret_manager_keys(secrets_json, &client_sec).await?;

            let aux = "";
            create_secret_manager_secret_key(aux, &client_sec).await?;
        }
    }
    match keys {
        None => {}
        Some(keys_ok) => {
            let client_key = aws_sdk_kms::client::Client::new(config.aws_config());
            create_key(&client_key).await?;
        }
    }

    Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "non-terraform dependencies creation/delete",
    about = "Create and Destroy dependencies"
)]
pub struct Opt {
    #[structopt(short = "t", long = "table")]
    pub table: Option<String>,

    #[structopt(short = "c", long = "create")]
    pub create: bool,

    #[structopt(short = "d", long = "delete")]
    pub delete: bool,

    #[structopt(env = "ENVIRONMENT")]
    pub environment: String,

    #[structopt(short = "s", long = "secrets")]
    pub secrets: Option<bool>,

    #[structopt(short = "k", long = "keys")]
    pub keys: Option<bool>,
}

#[tokio::main]
async fn main() {
    let op_cmd = command(Opt::from_args()).await;
    match op_cmd {
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
        Ok(_) => {
            println!("successful!")
        }
    }
}
