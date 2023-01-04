use actix_web::{
    body::EitherBody,
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use lib_users::models::user::UserRoles;
use std::future::{ready, Ready};

use super::jwt_middleware::get_header_jwt;

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

        let req_headers = req.headers();

        let claim_ops = get_header_jwt(req_headers);

        match claim_ops {
            Ok(clm) => {
                let matches = clm
                    .roles
                    .into_iter()
                    .map(|i| UserRoles::deserialize(i.as_str()).unwrap() )
                    .filter(|i| i.is_admin())
                    .count();
                if matches == 0 {
                    auth_flag = false;
                } else {
                    auth_flag = true;
                }
            }
            Err(e) => {
                let (request, _pl) = req.into_parts();
                let res = HttpResponse::Forbidden()
                    .body(e.to_string())
                    .map_into_right_body();

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
