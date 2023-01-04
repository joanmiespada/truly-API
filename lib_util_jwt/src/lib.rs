/*use actix_web::{
    body::EitherBody,
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{HeaderName , HeaderValue, AUTHORIZATION},
    Error, HttpRequest, HttpResponse, ResponseError,
};*/

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

pub fn create_jwt(
    uid: &str,
    roles: &Vec<String>,
    token_secret: &String,
) -> Result<String, JWTSecurityError> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
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

//pub fn check_jwt_token(request: &HttpRequest) -> Result<Claims, Error> {
pub fn check_jwt_token(token: &String, token_secret: &String) -> Result<Claims, JWTSecurityError> {
    /*
    let req_headers = request.headers();

    let header = match req_headers.get(AUTHORIZATION) {
        Some(v) => v,
        None =>
        {
            return Err(JWTSecurityError::from("jwt error: no auth header field".to_string()).into())
        }
    };
    let auth_header = match std::str::from_utf8(header.as_bytes()) {
        Ok(v) => v,
        Err(_) =>
        {
            return Err(JWTSecurityError::from("jwt error: no auth header field with value".to_string()).into())
        }
    };*/

    if !token.starts_with(BEARER) {
        return Err(JWTSecurityError::from("jwt error".to_string()).into());
    }
    let jwt = token.trim_start_matches(BEARER).to_owned();

    //let jwt_secret = std::env::var("JWT_TOKEN_BASE").unwrap();
    let decoded = decode::<Claims>(
        &jwt,
        &DecodingKey::from_secret(token_secret.as_bytes()),
        &Validation::new(Algorithm::HS512),
    );
    match decoded {
        Err(_) => {
            return Err(JWTSecurityError::from("token present but invalid".to_string()).into())
        }
        Ok(deco) => Ok(deco.claims),
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

/*
impl ResponseError for JWTSecurityError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.0.clone())
    }
}*/
