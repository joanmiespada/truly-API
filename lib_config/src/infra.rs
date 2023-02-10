use aws_sdk_kms::model::KeyUsageType;

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