use http::header::AUTHORIZATION;
use http::{HeaderMap, HeaderValue};

use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use lambda_http::Request;
use lib_config::config::Config;
use lib_users::models::user::UserRoles;
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
        write!(
            f,
            "uid: {}, roles: {:?}, expire: {}",
            self.uid, self.roles, self.exp
        )
    }
}

pub fn create_jwt(
    uid: &str,
    roles: Vec<String>,
    token_secret: &String,
    exp_hours: i64,
) -> Result<String, JWTSecurityError> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::hours(exp_hours))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        uid: uid.to_owned(),
        roles, //.clone(),
        exp: expiration as usize,
    };
    let header = Header::new(Algorithm::HS512);
    let key = EncodingKey::from_secret(token_secret.as_bytes());
    let jwt = encode(&header, &claims, &key);
    match jwt {
        Ok(x) => Ok(x),
        Err(_) => Err(JWTSecurityError::from("fail creating a token".to_string())),
    }
}

pub fn check_jwt_token(token: &str, token_secret: &String) -> Result<Claims, JWTSecurityError> {
    if !token.starts_with(BEARER) {
        return Err(JWTSecurityError::from("jwt error".to_string()));
    }
    let jwt = token.trim_start_matches(BEARER).to_owned();

    let decoded = decode::<Claims>(
        &jwt,
        &DecodingKey::from_secret(token_secret.as_bytes()),
        &Validation::new(Algorithm::HS512),
    );
    match decoded {
        Err(_) => Err(JWTSecurityError::from(
            "token present but invalid, login again".to_string(),
        )),
        Ok(deco) => Ok(deco.claims),
    }
}

#[derive(Debug)]
pub struct JWTSecurityError {
    message: String,
}

impl std::fmt::Display for JWTSecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "jwt error {:?}", self.message)
    }
}

impl From<String> for JWTSecurityError {
    fn from(err: String) -> JWTSecurityError {
        JWTSecurityError { message: err }
    }
}

pub fn get_header_jwt(
    req_headers: &HeaderMap<HeaderValue>,
    jwt_secret: &String,
) -> Result<Claims, JWTSecurityError> {
    match req_headers.get(AUTHORIZATION) {
        Some(header_v) => {
            match std::str::from_utf8(header_v.as_bytes()) {
                Ok(header_field_value) => {
                    //let jwt_secret =  config.env_vars().jwt_token_base();

                    let claim = check_jwt_token(header_field_value, jwt_secret);

                    match claim {
                        Ok(clm) => Ok(clm),
                        Err(e) => Err(e),
                    }
                }
                Err(_) => Err(JWTSecurityError::from(
                    "jwt error: no auth header field with value valid".to_string(),
                )),
            }
        }
        None => Err(JWTSecurityError::from(
            "jwt error: no auth header field present".to_string(),
        )),
    }
}

pub fn check_jwt_token_as_admin(req: &Request, config: &Config) -> Result<bool, JWTSecurityError> {
    let auth_flag;
    let req_headers = req.headers();

    let jwt_secret = config.env_vars().jwt_token_base().unwrap();
    let claim_ops = get_header_jwt(req_headers, &jwt_secret);
    match claim_ops {
        Ok(clm) => {
            log::info!("{}", clm.to_string());
            let matches = clm
                .roles
                .into_iter()
                .map(|i| UserRoles::deserialize(i.as_str()).unwrap())
                .filter(|i| i.is_admin())
                .count();
            if matches == 0 {
                auth_flag = false;
            } else {
                auth_flag = true;
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
    Ok(auth_flag)
}
