
use actix_web::{ Error};
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
