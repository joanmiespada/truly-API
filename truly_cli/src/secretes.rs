use std::{fs::File, io::{BufReader, Read}};

use aws_sdk_dynamodb::types::error::ResourceNotFoundException;
use lib_config::{config::Config, infra::create_secret_manager_keys};

pub async fn create_secrets(
    create: bool,
    delete: bool,
    secrets_json_path: String,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let er = ResourceNotFoundException::builder().build();
    if create {
        let client_sec = aws_sdk_secretsmanager::client::Client::new(config.aws_config());
        
        let file = File::open(secrets_json_path)?;
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();

        buf_reader.read_to_string(&mut contents)?;

        create_secret_manager_keys(&contents, &client_sec).await?;

    } else if delete {
        panic!("not allowed, do it with AWS console UI")
    } else {
        return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
    }

    Ok(())
}
