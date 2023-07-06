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

        if let Err(e) = create_secret_manager_with_values(&contents, config).await {
            error!("{}", e);
        }
        
    } else if delete {
        panic!("not allowed, do it with AWS console UI")
    } else {
        return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
    }

    Ok(())
}

// use lib_config::{
//     //config::Config,
//     secrets::{SECRETS_MANAGER_APP_KEYS},
// };

// const TAG_PROJECT: &str = "Project";
// const TAG_VALUE: &str = "Truly";
// const TAG_ENVIRONMENT: &str = "Environment";

// pub async fn create_secret_manager_with_values2(
//     secrets_json: &str,
//     config: &Config
// ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

//     let client = aws_sdk_secretsmanager::client::Client::new(config.aws_config());

//     let env = config.env_vars().environment().unwrap();

//     let op =client
//         .create_secret()
//         .name(SECRETS_MANAGER_APP_KEYS.to_string())
//         .secret_string(secrets_json)
//         .tags(
//             aws_sdk_secretsmanager::types::Tag::builder()
//                 .key(TAG_PROJECT.to_owned())
//                 .value(TAG_VALUE.to_owned())
//                 .build(),
//         )
//         .tags(
//             aws_sdk_secretsmanager::types::Tag::builder()
//                 .key(TAG_ENVIRONMENT.to_owned())
//                 .value( env )
//                 .build(),
//         )
//         .send()
//         .await;

//     match op {
//         Err(e) => {
//             error!("{}",e);
//             Err(Box::new(e) as Box<dyn  std::error::Error + Send + Sync>)
//         },
//         Ok(a) =>{
//             print!("{:?}",a);
//             Ok(())
//         }

//     }

// }
