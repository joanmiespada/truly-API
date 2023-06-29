use std::{fs::File, io::{BufReader, Read}};

use aws_sdk_dynamodb::types::error::ResourceNotFoundException;

use lib_config::{
    config::Config,
    infra::{create_secret_manager_secret_key, store_secret_key},
};

pub async fn create_store_key(
    key_id: String,
    create: bool,
    delete: bool,
    key_file_path: String,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let er = ResourceNotFoundException::builder().build();
    if create {
        let client_sec = aws_sdk_secretsmanager::client::Client::new(config.aws_config());
        // let secret_key_raw;
        // if environment == DEV_ENV {
        //     secret_key_raw = include_str!("../res/key_development.txt");
        // } else {
        //     secret_key_raw = include_str!("../res/key_prod_stage.txt");
        // }
        let file = File::open(key_file_path)?;
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();

        buf_reader.read_to_string(&mut contents)?;

        let res_op = create_secret_manager_secret_key(&client_sec).await;
        match res_op {
            Err(e) => {
                panic!("{}", e.to_string())
            }
            Ok(_) => match store_secret_key(&contents, &key_id, &config).await {
                Err(e) => panic!("{}", e.to_string()),
                Ok(_) => {}
            },
        }
    } else if delete {
        panic!("not allowed, do it with AWS console UI")
    } else {
        return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
    }
    Ok(())
}
