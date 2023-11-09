use lib_config::secrets::SECRETS_MANAGER_APP_KEYS;

pub async fn create_secrets(
    client: &aws_sdk_secretsmanager::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let secrets_json = r#"
        {
            "HMAC_SECRET" : "localtest_hmac_fgsdfg3rterfr2345weg@#$%WFRsdf",
            "JWT_TOKEN_BASE": "localtest_jwt_fdgsdfg@#$%Sdfgsdfg@#$3",
            "PAGINATION_TOKEN":"test_token_pagination"
        }
        "#;

    client
        .create_secret()
        .name(SECRETS_MANAGER_APP_KEYS.to_string())
        .secret_string(secrets_json)
        .send()
        .await?;

    Ok(())
}
