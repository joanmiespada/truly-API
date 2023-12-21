use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use lib_config::result::ResultE;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use url::Url;
use crate::ExternalData;
use self::error::InstagramAPIError;

pub mod error;

const BASE_URL: &str = "https://graph.instagram.com/";
const PAGE_SIZE: i32 = 10;

#[derive(Deserialize)]
struct OAuthResponse {
    access_token: String,
    //token_type: String,
    expires_in: u64, // Duration in seconds
}

#[derive(Clone, Debug)]
pub struct InstagramAPI {
    access_token: Option<String>,
    token_expiration: Option<DateTime<Utc>>,
    // Include other fields as needed
}

impl InstagramAPI {
    pub fn new(access_token: Option<String>, token_expiration: Option<DateTime<Utc>>) -> InstagramAPI {
        InstagramAPI {
            access_token,
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
        // Implement the logic to refresh or re-authenticate to get a new access token
        // This typically involves making a request to the OAuth token endpoint
        let refresh_token_endpoint = format!(
            "refresh_access_token?grant_type=ig_refresh_token&access_token={}", 
            self.access_token.as_ref().ok_or_else(|| InstagramAPIError("Missing token".to_string()))?
        );

        let res = self.query(&refresh_token_endpoint).await?;
        if res.status().is_success() {
            let response_body = res.json::<OAuthResponse>()
                .await
                .map_err(|e| InstagramAPIError(format!("Failed to parse response: {}", e)))?;

            // Update the token and expiration
            self.access_token = Some(response_body.access_token);
            // Set the appropriate expiration based on the expires_in field
            self.token_expiration = Some(Utc::now() + Duration::seconds(response_body.expires_in as i64));

            Ok(())
        } else {
            // Handle non-successful responses
            Err(InstagramAPIError("Failed to refresh token".to_string()).into() )
        }

    }

    async fn query(&self, endpoint: &str) -> ResultE<reqwest::Response> {
        let client = Client::new();
        let url = format!("{}{}", BASE_URL, endpoint);

        client
            .get(&url)
            //.bearer_auth(self.access_token.as_ref().ok_or_else(|| InstagramAPIError("Missing token".to_string()))?)
            .send()
            .await
            .map_err(|e| InstagramAPIError(format!("Failed to send request: {}", e)).into() )
    }

    // Add methods to interact with specific endpoints, e.g., to get recent videos
}

#[async_trait]
impl ExternalData for InstagramAPI {
    async fn search(&mut self, word_keys: Vec<String>, page_token: Option<String>, last24h:bool) -> ResultE<(Vec<Url>, Option<String>)>
    {
        self.ensure_token_valid().await?;

        let mut endpoint = format!("me/media?fields=id,caption,media_type,thumbnail_url,permalink,media_url&limit={}&q={}",PAGE_SIZE, word_keys.join("+"));

        // Optionally add parameters like 'page_token' and 'last24h' to the endpoint if needed
        if let Some(token) = page_token {
            endpoint.push_str(&format!("&page_token={}", token));
        }

        if last24h {
            let start_time = Utc::now() - Duration::hours(24);
            let start_time_str = start_time.to_rfc3339();

            // Add the date range parameters to the endpoint
            endpoint.push_str(&format!(
                "&timestamp={}-{}",
                start_time_str,
                Utc::now().to_rfc3339()
            ));
        }

        // Make the API request
        let response = self.query(&endpoint).await?;
        
        if response.status().is_success() {
            // Parse the JSON response
            let response_body: Value = response.json().await?;
            
            // Extract video URLs from the response JSON
            let mut video_urls = vec![];
            if let Some(data) = response_body.get("data").and_then(|data| data.as_array()) {
                for entry in data {
                    if let Some(media_url) = entry.get("media_url").and_then(|url| url.as_str()) {
                        video_urls.push(Url::parse(media_url)?);
                    }
                }
            }

            // Check for a 'next' page token if you're paginating through results
            let next_page_token = response_body
                .get("paging")
                .and_then(|paging| paging.get("next"))
                .and_then(|next| next.as_str())
                .map(String::from);

            Ok((video_urls, next_page_token))
        } else {
            Err("Failed to fetch search results".into())
        }

    }

    async fn search_by_category(&mut self, word_keys: Vec<String>, page_token: Option<String>, last24h:bool) -> ResultE<(Vec<Url>, Option<String>)>
    {
        self.ensure_token_valid().await?;
        Ok(( Vec::new(), None))

    }

    // Implement other required methods
}
