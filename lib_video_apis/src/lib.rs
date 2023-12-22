use async_trait::async_trait;
use lib_config::result::ResultE;
use url::Url;

pub mod youtube;
pub mod twitch;
pub mod instagram;
pub mod facebook;
pub mod twitter;
pub mod runner;

#[async_trait]
pub trait ExternalData {
    async fn search(&mut self, word_keys: Vec<String>, page_token: Option<String>, last24h:bool) -> ResultE<(Vec<Url>, Option<String>)>;
    async fn search_by_category(&mut self, word_keys: Vec<String>, page_token: Option<String>, last24h:bool) -> ResultE<(Vec<Url>, Option<String>)>;
}