
use crate::http::handlers;
use actix_web::{dev::Server, web, App, HttpServer, Error};
use async_trait::async_trait;

#[async_trait]
pub trait Starter {
    async fn start(&self) -> Result<(),Error>;
}

pub struct Http;

#[async_trait]
impl Starter for Http {
    async fn start(&self) -> Result<(),Error> {

        // TODO
        
        Ok(())
    }
}
