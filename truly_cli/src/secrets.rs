use std::{
    fs::File,
    io::{BufReader, Read},
};
use log::error;

use aws_sdk_dynamodb::types::error::ResourceNotFoundException;
use lib_config::config::Config;
use lib_config::infra::create_secret_manager_with_values;

pub async fn create_secrets(
    create: bool,
    delete: bool,
    secret_id: String,
    secrets_json_path: String,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let er = ResourceNotFoundException::builder().build();
    if create {
        //let client_sec = aws_sdk_secretsmanager::client::Client::new(config.aws_config());

        let file = File::open(secrets_json_path)?;
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();

        buf_reader.read_to_string(&mut contents)?;

        if let Err(e) = create_secret_manager_with_values(&contents, &secret_id, config).await {
            error!("{}", e);
        }
        
    } else if delete {
        panic!("not allowed, do it with AWS console UI")
    } else {
        return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
    }

    Ok(())
}

