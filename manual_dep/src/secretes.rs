use lib_config::{config::Config, environment::DEV_ENV, infra::create_secret_manager_keys};
use aws_sdk_dynamodb::types::error::ResourceNotFoundException;

pub async fn create_secrets(
    create: bool,
    delete: bool,
    environment: String,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let er = ResourceNotFoundException::builder().build();
    if create {
        let client_sec = aws_sdk_secretsmanager::client::Client::new(config.aws_config());
        let secrets_json;
        if environment == DEV_ENV {
            secrets_json = include_str!("../res/secrets_development.json");
        } else {
            secrets_json = include_str!("../res/secrets_prod_stage.json");
        }
        create_secret_manager_keys(secrets_json, &client_sec).await?;
    } else if delete {
        panic!("not allowed, do it with AWS console UI")
    } else {
        return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
    }

    Ok(())
}
