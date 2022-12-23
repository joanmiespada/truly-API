use actix_web::{
    body::EitherBody,
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    http::{header::AUTHORIZATION},
    Error, HttpRequest, HttpResponse, ResponseError,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use std::future::{ready, Ready};

use crate::handlers::jwt_middleware::{ JWTSecurityError, BEARER };

use super::login_hd::Claims;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct Auth;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware { service }))
    }
}

pub struct AuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let mut auth_flag = false;

        //println!("Hi from start. You requested: {}", req.path());

        let aux = checkRoles(req.request());

        match aux {
            Ok(_) => {
                auth_flag = true;
            }
            Err(e) => {
                let (request, _pl) = req.into_parts();
                let res = HttpResponse::Forbidden().finish().map_into_right_body();

                let new_response = ServiceResponse::new(request, res);
                return Box::pin(async move { Ok(new_response) });
            }
        }

        if auth_flag {
            let fut = self.service.call(req);

            Box::pin(async move {
                // let res = fut.await?;
                fut.await.map(ServiceResponse::map_into_left_body)

                //println!("Hi from response");
                //Ok(res)
            })
        } else {
            let (request, _pl) = req.into_parts();
            let res = HttpResponse::Forbidden().finish().map_into_right_body();

            let new_response = ServiceResponse::new(request, res);
            Box::pin(async move { Ok(new_response) })
        }
    }
}

pub fn checkRoles(request: &HttpRequest) -> Result<bool, Error> {
    let req_headers = request.headers();
    //let basic_auth_header = req_headers.get(AUTHORIZATION);

    let header = match req_headers.get(AUTHORIZATION) {
        Some(v) => v,
        None =>
        // Err(Error::NoAuthHeaderError),
        {
            return Err(JWTSecurityError::from("jwt error".to_string()).into())
        }
    };
    let auth_header = match std::str::from_utf8(header.as_bytes()) {
        Ok(v) => v,
        Err(_) =>
        // Err(Error::NoAuthHeaderError),
        {
            return Err(JWTSecurityError::from("jwt error".to_string()).into())
        }
    };
    if !auth_header.starts_with(BEARER) {
        return Err(JWTSecurityError::from("jwt error".to_string()).into());
    }
    let jwt = auth_header.trim_start_matches(BEARER).to_owned();

    //TODO: reading from env_vars instead of config
    let jwt_secret = std::env::var("JWT_TOKEN_BASE").unwrap();
    let decoded = decode::<Claims>(
        &jwt,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS512),
    );
    match decoded {
        Err(e) => return Err(JWTSecurityError::from("jwt error".to_string()).into()),
        Ok(deco) => {
            let matches = deco
                .claims
                .roles
                .into_iter()
                .filter(|i| i.is_admin())
                .count();
            if matches == 0 {
                return Err(JWTSecurityError::from("jwt error".to_string()).into());
            }

            Ok(true)
        }
    }
}

/*
#[derive(Debug)]
pub enum MyErrorTypes {
    //#[error("wrong credentials")]
    WrongCredentialsError,
    //#[error("jwt token not valid")]
    JWTTokenError,
    //#[error("jwt token creation error")]
    JWTTokenCreationError,
    //#[error("no auth header")]
    NoAuthHeaderError,
    //#[error("invalid auth header")]
    InvalidAuthHeaderError,
    //#[error("no permission")]
    NoPermissionError,
    OtherError(String),
}
*/

// #[derive(Debug)] //, Display, Error)]
//                  //#[display(fmt = "my error: {}", name)]
// pub struct JWTSecurityError {
//     pub name: Option<String>,
//     //pub err_type: MyErrorTypes,
// }

// impl JWTSecurityError {
//     pub fn message(&self) -> String {
//         match &self.name {
//             Some(c) => c.clone(),
//             None => String::from(""),
//         }
//     }
// }

// impl std::fmt::Display for JWTSecurityError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }

// impl From<String> for JWTSecurityError {
//     fn from(err: String) -> JWTSecurityError {
//         JWTSecurityError {
//             name: Some(err),
//             //err_type: MyErrorTypes::OtherError("ssdfd".to_string()),
//         }
//     }
// }

// impl ResponseError for JWTSecurityError {
//     /*fn status_code(&self) -> StatusCode {
//         match self.err_type {
//             JWTTokenCreationError => StatusCode::INTERNAL_SERVER_ERROR,
//             WrongCredentialsError => StatusCode::BAD_REQUEST,
//             JWTTokenError => StatusCode::INTERNAL_SERVER_ERROR,
//             NoAuthHeaderError => StatusCode::INTERNAL_SERVER_ERROR,
//         }
//     }*/

//     fn error_response(&self) -> HttpResponse {
//         HttpResponse::build(self.status_code()).json(self.name.clone())
//     }
// }
