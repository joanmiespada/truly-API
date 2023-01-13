
use http::{HeaderMap, HeaderValue};
use http::header::AUTHORIZATION;

use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

pub const BEARER: &str = "Bearer ";

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub uid: String,
    pub roles: Vec<String>,
    pub exp: usize,
}

impl std::fmt::Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "uid: {}, roles: {:?}, expire: {}", self.uid, self.roles, self.exp)
    }
}

pub fn create_jwt(
    uid: &str,
    roles: &Vec<String>,
    token_secret: &String,
    exp_hours: i64
) -> Result<String, JWTSecurityError> {

    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::hours(exp_hours))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        uid: uid.to_owned(),
        roles: roles.clone(),
        exp: expiration as usize,
    };
    let header = Header::new(Algorithm::HS512);
    let key = EncodingKey::from_secret(token_secret.as_bytes());
    let jwt = encode(&header, &claims, &key);
    match jwt {
        Ok(x) => Ok(x),
        Err(_) => Err(JWTSecurityError::from("fail creating a token".to_string()).into()),
    }
}

pub fn check_jwt_token(token: &String, token_secret: &String) -> Result<Claims, JWTSecurityError> {

    if !token.starts_with(BEARER) {
        return Err(JWTSecurityError::from("jwt error".to_string()).into());
    }
    let jwt = token.trim_start_matches(BEARER).to_owned();

    let decoded = decode::<Claims>(
        &jwt,
        &DecodingKey::from_secret(token_secret.as_bytes()),
        &Validation::new(Algorithm::HS512),
    );
    match decoded {
        Err(_) => {
            return Err(JWTSecurityError::from("token present but invalid, login again".to_string()).into())
        }
        Ok(deco) => {

            return Ok(deco.claims);
        }
    }
}

#[derive(Debug)]
pub struct JWTSecurityError(String);

impl std::fmt::Display for JWTSecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "jwt error {:?}", self.0)
    }
}

impl From<String> for JWTSecurityError {
    fn from(err: String) -> JWTSecurityError {
        JWTSecurityError { 0: err }
    }
}

pub fn get_header_jwt(
    req_headers: &HeaderMap<HeaderValue>,
    jwt_secret: &String
) -> Result<Claims, JWTSecurityError> {
    match req_headers.get(AUTHORIZATION) {
        Some(header_v) => {
            match std::str::from_utf8(header_v.as_bytes()) {
                Ok(header_field_value) => {
                    //let jwt_secret =  config.env_vars().jwt_token_base();

                    let claim = check_jwt_token(&header_field_value.to_string(), &jwt_secret);

                    match claim {
                        Ok(clm) => {
                            Ok(clm)
                        }
                        Err(e) => Err(e),
                    }
                }
                Err(_) => Err(JWTSecurityError::from("jwt error: no auth header field with value valid".to_string()).into()),
            }
        }
        None => Err(JWTSecurityError::from("jwt error: no auth header field present".to_string()).into()) ,
    }
}