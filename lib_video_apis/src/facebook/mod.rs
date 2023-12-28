pub mod error;

use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use lib_config::result::ResultE;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use url::Url;

use crate::ExternalData;

use self::error::FacebookAPIError;

const BASE_URL: &str = "https://graph.facebook.com/v13.0/"; // Use the appropriate API version
const PAGE_SIZE: i32 = 10;

#[derive(Deserialize)]
struct OAuthResponse {
    access_token: String,
    expires_in: u64, // Duration in seconds
    // Other fields you need
}

#[derive(Clone, Debug)]
pub struct FacebookAPI {
    access_token: Option<String>,
    token_expiration: Option<DateTime<Utc>>,
}

impl FacebookAPI {
    
    pub fn new(access_token: Option<String>, token_expiration: Option<DateTime<Utc>>) -> FacebookAPI {
        FacebookAPI {
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
            "refresh_access_token?grant_type=fb_refresh_token&access_token={}",
             self.access_token.as_ref().ok_or_else(|| FacebookAPIError("Missing token".to_string()) )?
        );
        let res = self.query(&refresh_token_endpoint).await?;
        if res.status().is_success() {
            let response_body = res.json::<OAuthResponse>()
                .await
                .map_err(|e| FacebookAPIError(format!("Failed to parse response: {}", e)))?;

            // Update the token and expiration
            self.access_token = Some(response_body.access_token);
            // Set the appropriate expiration based on the expires_in field
            self.token_expiration = Some(Utc::now() + Duration::seconds(response_body.expires_in as i64));

            Ok(())
        } else {
            // Handle non-successful responses
            Err(FacebookAPIError("Failed to refresh token".to_string()).into() )
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
            .map_err(|e| FacebookAPIError(format!("Failed to send request: {}", e)).into() )
    }

}

#[async_trait]
impl ExternalData for FacebookAPI {


    async fn search(
        &mut self,
        keywords: Vec<String>,
        page_token: Option<String>,
        last24h: bool,
    ) -> ResultE<(Vec<Url>,Option<String>)> {
        self.ensure_token_valid().await?;

        // Construct the search endpoint
        let mut endpoint = format!("search?type=post&limit={}&q={}", PAGE_SIZE, keywords.join("+"));

        // Optionally add parameters like 'page_token' and 'last24h' to the endpoint if needed
        if let Some(token) = page_token {
            endpoint.push_str(&format!("&after={}", token));
        }

        if last24h {
            let start_time = Utc::now() - Duration::hours(24);
            let start_time_str = start_time.to_rfc3339();

            // Add the date range parameters to the endpoint
            endpoint.push_str(&format!(
                "&created_time_since={}",
                start_time_str
            ));
        }

        // Make the API request
        let response = self.query(&endpoint).await?;

        if response.status().is_success() {
            // Parse the JSON response
            let response_body: Value = response.json().await?;

            // Extract URLs of posts with video media type from the response JSON
            let mut video_post_urls = Vec::new();
            if let Some(data) = response_body.get("data").and_then(|data| data.as_array()) {
                for entry in data {
                    // Check the media type of the post
                    if let Some(media_type) = entry.get("type").and_then(|t| t.as_str()) {
                        if media_type == "video" {
                            // Extract the permalink URL
                            if let Some(permalink) = entry.get("permalink_url").and_then(|p| p.as_str()) {
                                video_post_urls.push(Url::parse(permalink)?);
                            }
                        }
                    }
                }
            }

            let next_page_token = response_body
                .get("paging")
                .and_then(|paging| paging.get("next"))
                .and_then(|next| next.as_str())
                .map(String::from);

            Ok((video_post_urls,next_page_token))
        } else {
            Err("Failed to fetch search results".into())
        }
    }

    //TODO
    async fn search_by_category(&mut self, _word_keys: Vec<String>, _page_token: Option<String>, _last24h:bool) -> ResultE<(Vec<Url>, Option<String>)>
    {
        self.ensure_token_valid().await?;
        Ok(( Vec::new(), None))

    }

    // Implement other required methods
}