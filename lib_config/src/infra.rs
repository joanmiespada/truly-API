use aws_config::meta::region::RegionProviderChain;
use aws_config::SdkConfig;
use aws_sdk_dynamodb::{primitives::Blob, config::Credentials};
use aws_sdk_kms::types::KeyUsageType;
use base64::{engine::general_purpose, Engine as _};

use crate::{
    config::Config,
    secrets::{SECRETS_MANAGER_APP_KEYS, SECRETS_MANAGER_SECRET_KEY},
};

const TAG_PROJECT: &str = "Project";
const TAG_VALUE: &str = "Truly";

pub async fn create_secret_manager_keys(
    secrets_json: &str,
    client: &aws_sdk_secretsmanager::Client,
) -> Result<(), Box<dyn std::error::Error + Send +Sync>> {
    client
        .create_secret()
        .name(SECRETS_MANAGER_APP_KEYS.to_string())
        .secret_string(secrets_json)
        .tags(
            aws_sdk_secretsmanager::types::Tag::builder()
                .key(TAG_PROJECT.to_owned())
                .value(TAG_VALUE.to_owned())
                .build(),
        )
        .send()
        .await?;

    Ok(())
}

pub async fn create_secret_manager_secret_key(
    client: &aws_sdk_secretsmanager::Client,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let aux = client
        .create_secret()
        .name(SECRETS_MANAGER_SECRET_KEY.to_string())
        .secret_string("--")
        .tags(
            aws_sdk_secretsmanager::types::Tag::builder()
                .key(TAG_PROJECT.to_owned())
                .value(TAG_VALUE.to_owned())
                .build(),
        )
        .send()
        .await;
    match aux {
        Err(e) => panic!("{}",e.to_string()),
        Ok(_) => Ok(())
    }
}

pub async fn create_key(
    client: &aws_sdk_kms::Client,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let resp = client
        .create_key()
        .description("key used to encryp private key for contract owner")
        .key_usage(KeyUsageType::EncryptDecrypt)
        .tags(
            aws_sdk_kms::types::Tag::builder() 
                .tag_key(TAG_PROJECT.to_owned())
                .tag_value(TAG_VALUE.to_owned())
                .build(),
        )
        .send()
        .await?;
    //let id = resp.key_metadata.unwrap().arn().unwrap().to_string();
    let id = resp.key_metadata.unwrap().key_id().unwrap().to_string();

    Ok(id)
}

pub async fn build_local_stack_connection(host_port: u16) -> SdkConfig {
    let endpoint_url = format!("http://127.0.0.1:{}", host_port);
    //let uri = Uri::from_str(&endpoint_uri).unwrap();
    //let endpoint_resolver = Endpoint::immutable_uri(uri);
    let region_provider = RegionProviderChain::default_provider().or_else("eu-central-1");
    let creds = Credentials::new("test", "test", None, None, "test");

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

pub async fn store_secret_key(
    info_to_be_encrypted: &str,
    kms_key_id: &str,
    config: &Config,
) -> Result<String, Box<dyn std::error::Error + Send + Sync >> {
    let aws = config.aws_config();

    let client = aws_sdk_kms::Client::new(aws);

    let blob = Blob::new(info_to_be_encrypted.as_bytes());
    let resp_op = client
        .encrypt()
        .key_id(kms_key_id.clone())
        .plaintext(blob)
        .send()
        .await;
    let resp = resp_op.unwrap();

    let blob = resp.ciphertext_blob.expect("Could not get encrypted text");
    let bytes = blob.as_ref();

    let value = general_purpose::STANDARD.encode(bytes);

    Ok(value)
}

pub async fn restore_secret_key(
    info_to_be_decyphered: String,
    kms_key_id: &str,
    config: &Config,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let aws = config.aws_config();

    let value = general_purpose::STANDARD
        .decode(info_to_be_decyphered)
        .unwrap();

    let client2 = aws_sdk_kms::Client::new(&aws);

    let data = aws_sdk_kms::primitives::Blob::new(value);

    let resp;
    let resp_op = client2
        .decrypt()
        .key_id(kms_key_id.to_owned())
        .ciphertext_blob(data.to_owned())
        .send()
        .await;
    match resp_op {
        Err(e) => {
            return Err(e.into());
        }
        Ok(val) => resp = val,
    }

    let inner = resp.plaintext.unwrap();
    let bytes = inner.as_ref();

    let secret_key_raw = String::from_utf8(bytes.to_vec()).unwrap(); // .expect("Could not convert to UTF-8");

    Ok(secret_key_raw)
}
