
use serde::Deserialize;


pub const SECRETS_MANAGER_APP_KEYS: &str = "truly_app_keys";
pub const SECRETS_MANAGER_SECRET_KEY: &str = "truly_contract_owners_secret_key";

#[derive(Deserialize, Debug)]
pub struct Secrets {
    #[serde(rename = "HMAC_SECRET")] //candidate to be removed from here and use the KMS_ID
    pub hmac_secret: String,
    #[serde(rename = "JWT_TOKEN_BASE")]
    pub jwt_token_base: String,
    #[serde(rename = "BLOCKCHAIN_GATEWAY_API_KEY")]
    pub blockchain_gateway_api_key: String,
}
