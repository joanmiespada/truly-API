use aws_sdk_kms::model::KeyUsageType;
use aws_config::SdkConfig;
use aws_sdk_dynamodb::Credentials;
use aws_config::meta::region::RegionProviderChain;

use crate::secrets::{SECRETS_MANAGER_KEYS, SECRETS_MANAGER_SECRET_KEY};


pub async fn create_secret_manager_keys(
    secrets_json: &str,
    client: &aws_sdk_secretsmanager::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    
    // let secrets_json = r#"
    // {
    //     "HMAC_SECRET" : "localtest_hmac_1234RGsdfg#$%",
    //     "JWT_TOKEN_BASE": "localtest_jwt_sd543ERGds235$%^"
    // }
    // "#;

    client
        .create_secret()
        .name(SECRETS_MANAGER_KEYS.to_string())
        .secret_string(  secrets_json  )
        .send()
        .await?;

    Ok(())
}

pub async fn create_secret_manager_secret_key(
    content: &str,
    client: &aws_sdk_secretsmanager::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .create_secret()
        .name(SECRETS_MANAGER_SECRET_KEY.to_string())
        .secret_string(content  )
        .send()
        .await?;
    Ok(())
}

pub async fn create_key(client: &aws_sdk_kms::Client) -> Result<String, Box<dyn std::error::Error>> {
    let resp =client
        .create_key()
        .key_usage(KeyUsageType::EncryptDecrypt)
        .send()
        .await?;
    let id = resp
        .key_metadata
        .unwrap()
        .key_id
        .unwrap();

    Ok(id)
}



pub async fn build_local_stack_connection(host_port: u16) -> SdkConfig {
    let endpoint_url = format!("http://127.0.0.1:{}", host_port);
    //let uri = Uri::from_str(&endpoint_uri).unwrap();
    //let endpoint_resolver = Endpoint::immutable_uri(uri);
    let region_provider = RegionProviderChain::default_provider().or_else("eu-central-1");
    let creds = Credentials::new(
        "test", 
        "test", 
        None, 
        None, 
        "test");

    let shared_config = aws_config::from_env()
        .region(region_provider)
        .endpoint_url(endpoint_url)
        //.endpoint_resolver(endpoint_resolver.unwrap())
        .credentials_provider(creds)
        .load()
        .await;

    //Client::new(&shared_config)
    return shared_config;
}