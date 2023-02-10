
use serde::Deserialize;


pub const SECRETS_MANAGER_KEYS: &str = "truly/api/secrets";
pub const SECRETS_MANAGER_SECRET_KEY: &str = "truly/api/secret_key";

#[derive(Deserialize, Debug)]
pub struct Secrets {
    #[serde(rename = "HMAC_SECRET")]
    pub hmac_secret: String,
    #[serde(rename = "JWT_TOKEN_BASE")]
    pub jwt_token_base: String,
}
