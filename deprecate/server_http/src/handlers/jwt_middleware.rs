use actix_web::{
    body::EitherBody,
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION},
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

pub const UID_HEAD_KEY: &str = "api-user-uid";

use lib_util_jwt::{check_jwt_token, Claims};

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct Jwt; //{
                //environment_vars: EnvironmentVariables
                //}

//impl Jwt{
//    pub fn new() -> Jwt {
//        Jwt {
// environment_vars: cnf.env_vars().clone(),
//        }
//    }
//}

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for Jwt
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddleware { service }))
    }
}

pub struct JwtMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for JwtMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let mut jwt_token = false;
        let mut uid: Option<String> = None;
        let mut error_message = "".to_string();
        //println!("Hi from start. You requested: {}", req.path());

        let req_headers = req.headers();

        let claim_ops = get_header_jwt(req_headers);
        match claim_ops {
            Err(e) => error_message = e.clone(),
            Ok(claims) => {
                jwt_token = true;
                uid = Some(claims.uid);
            }
        }

        if jwt_token {
            //inyect as header the user ID

            let head_value = HeaderValue::from_str(uid.unwrap().as_str()).unwrap();
            let head_key = HeaderName::from_static(UID_HEAD_KEY);
            req.headers_mut().append(head_key, head_value);
            let fut = self.service.call(req);

            Box::pin(async move {
                // let res = fut.await?;
                fut.await.map(ServiceResponse::map_into_left_body)

                //println!("Hi from response");
                //Ok(res)
            })
        } else {
            let (request, _pl) = req.into_parts();
            let res = HttpResponse::Forbidden()
                .body(format!("not allowed, login first: {}", error_message))
                .map_into_right_body();

            let new_response = ServiceResponse::new(request, res);
            return Box::pin(async move { Ok(new_response) });
        }
    }
}

pub fn get_header_jwt(req_headers: &HeaderMap) -> Result<Claims, String> {
    match req_headers.get(AUTHORIZATION) {
        Some(header_v) => {
            match std::str::from_utf8(header_v.as_bytes()) {
                Ok(header_field_value) => {
                    let jwt_secret = std::env::var("JWT_TOKEN_BASE").unwrap(); //TODO! Remove it here and inject config!!!
                                                                               //let jwt_secret = .unwrap(); //TODO! Remove it here and inject config!!!

                    let claim = check_jwt_token(&header_field_value.to_string(), &jwt_secret);

                    match claim {
                        Ok(clm) => {
                            Ok(clm)
                            //jwt_token = true;
                            //uid = Some(clm.uid);
                        }
                        Err(e) => Err(e.to_string()),
                    }
                }
                Err(_) => Err("jwt error: no auth header field with value valid".to_string()),
            }
        }
        None => Err("jwt error: no auth header field present".to_string()),
    }
}
