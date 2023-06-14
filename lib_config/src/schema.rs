use async_trait::async_trait;
use crate::config::Config;
use crate::result::ResultE;

#[async_trait]
pub trait Schema {
    async fn create_schema(config: &Config) -> ResultE<()>;
    async fn delete_schema(config: &Config) -> ResultE<()>;
}