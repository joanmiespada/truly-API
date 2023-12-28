pub mod error;

use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use derive_builder::Builder;
use lib_config::result::ResultE;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use url::Url;
use base64::{engine::general_purpose, Engine};

use crate::ExternalData;

use self::error::TwitterAPIError;

const BASE_URL: &str = "https://api.twitter.com/2/"; // Use the appropriate API version
const PAGE_SIZE: i32 = 10;

#[derive(Deserialize)]
struct OAuthResponse {
    access_token: String,
    expires_in: u64,
    //token_type: String,
}

#[derive(Clone, Debug,Builder)]
pub struct TwitterAPI {
    api_key: String,
    api_secret: String,
    #[builder(default)]
    bearer_token: Option<String>,
    #[builder(default)]
    token_expiration: Option<DateTime<Utc>>,
}

impl TwitterAPI {
    pub fn new( api_key: String, api_secret: String,access_token: Option<String>, token_expiration: Option<DateTime<Utc>>) -> TwitterAPI {
        TwitterAPI {
            api_key,
            api_secret,
            bearer_token: access_token,
            token_expiration,
        }
    }

    async fn ensure_token_valid(&mut self) -> ResultE<()> {
        if let Some(expiration) = self.token_expiration {
            if Utc::now() < expiration {
                return Ok(());
            }
        }
        self.refresh_oauth_token().await?;
        Ok(())
    }

    async fn refresh_oauth_token(&mut self) -> ResultE<()> {
        let credentials = general_purpose::STANDARD.encode(format!("{}:{}", self.api_key, self.api_secret));
        let client = Client::new();
        let params = [("grant_type", "client_credentials")];
        
        let res = client
            .post("https://api.twitter.com/oauth2/token")
            .header("Authorization", format!("Basic {}", credentials))
            .form(&params)
            .send()
            .await
            .map_err(|e| TwitterAPIError(format!("Failed to send OAuth token request: {}", e)))?;

        if res.status().is_success() {
            let response_body = res.json::<OAuthResponse>()
                .await
                .map_err(|e| TwitterAPIError(format!("Failed to parse OAuth response: {}", e)))?;

            self.bearer_token = Some(response_body.access_token);
            self.token_expiration = Some(Utc::now() + Duration::seconds(response_body.expires_in as i64));

            Ok(())
        } else {
            Err(TwitterAPIError("Failed to refresh token".to_string()).into())
        } 
    }


    async fn query(&self, endpoint: &str) -> ResultE<reqwest::Response> {
        let client = Client::new();
        let url = format!("{}{}", BASE_URL, endpoint);

        client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.bearer_token.as_ref().ok_or_else(|| TwitterAPIError("Missing token".to_string()))?))
            .send()
            .await
            .map_err(|e| TwitterAPIError(format!("Failed to send request: {}", e)).into())
    }
}

#[async_trait]
impl ExternalData for TwitterAPI {

    async fn search(
        &mut self,
        keywords: Vec<String>,
        page_token: Option<String>,
        last24h: bool,
    ) -> ResultE<(Vec<Url>, Option<String>)> {
        self.ensure_token_valid().await?;

        let query = keywords.join(" OR ");
        let mut endpoint = format!(
            "tweets/search/recent?query={}&max_results={}",
            query,
            PAGE_SIZE
        );

        if let Some(token) = page_token {
            endpoint.push_str(&format!("&pagination_token={}", token));
        }

        if last24h {
            let now = Utc::now();
            let yesterday = now - Duration::days(1);
            endpoint.push_str(&format!("&start_time={}", yesterday.to_rfc3339()));
        }

        let response = self.query(&endpoint).await?;

        if response.status().is_success() {
            let response_body: Value = response.json().await?;
            let tweets = response_body["data"].as_array().ok_or_else(|| TwitterAPIError("Invalid response format".to_string()))?;

            let tweet_urls = tweets.iter().filter_map(|tweet| {
                tweet["id"].as_str().map(|id| {
                    Url::parse(&format!("https://twitter.com/twitter/status/{}",id)).unwrap()
                })
            }).collect::<Vec<_>>();

            let next_page_token = response_body["meta"]["next_token"].as_str().map(String::from);

            Ok((tweet_urls, next_page_token))
        } else {
            Err(TwitterAPIError("Failed to fetch search results".to_string()).into())
        }
    }
    

    //TODO:
    async fn search_by_category(
        &mut self,
        _word_keys: Vec<String>,
        _page_token: Option<String>,
        _last24h: bool,
    ) -> ResultE<(Vec<Url>, Option<String>)> {
        self.ensure_token_valid().await?;
        Ok((Vec::new(), None))
    }

    // Implement other required methods
}
