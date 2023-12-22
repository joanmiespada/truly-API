// pub mod error;

// use async_trait::async_trait;
// use chrono::{DateTime, Utc, Duration};
// use lib_config::result::ResultE;
// use reqwest::Client;
// use serde::{Deserialize, Serialize};
// use serde_json::Value;
// use url::Url;

// use crate::ExternalData;

// use self::error::TwitterAPIError;

// const BASE_URL: &str = "https://api.twitter.com/2/"; // Use the appropriate API version
// const PAGE_SIZE: i32 = 10;

// #[derive(Deserialize)]
// struct OAuthResponse {
//     access_token: String,
//     token_type: String,
// }

// #[derive(Clone, Debug)]
// pub struct TwitterAPI {
//     api_key: String,
//     api_secret: String,
//     bearer_token: Option<String>,
//     token_expiration: Option<DateTime<Utc>>,
// }

// impl TwitterAPI {
//     pub fn new( api_key: String, api_secret: String,access_token: Option<String>, token_expiration: Option<DateTime<Utc>>) -> TwitterAPI {
//         TwitterAPI {
//             api_key,
//             api_secret,
//             bearer_token: access_token,
//             token_expiration,
//         }
//     }

//     async fn ensure_token_valid(&mut self) -> ResultE<()> {
//         if let Some(expiration) = self.token_expiration {
//             if Utc::now() < expiration {
//                 return Ok(());
//             }
//         }
//         self.refresh_oauth_token().await?;
//         Ok(())
//     }

//     async fn refresh_oauth_token(&mut self) -> ResultE<()> {
//         let bearer_endpoint = format!(
//             "https://api.twitter.com/oauth2/token"
//         );
//         let client = Client::new();
        
//         let res = client
//             .get(&bearer_endpoint)
//             .send()
//             .await
//             .map_err(|e| TwitterAPIError(format!("Failed to send get bearer request: {}", e)).into())?;
//         if res.status().is_success() {
//             let response_body = res.json::<OAuthResponse>()
//                 .await
//                 .map_err(|e| TwitterAPIError(format!("Failed to parse response: {}", e)))?;

//             // Update the token and expiration
//             self.bearer_token = Some(response_body.access_token);
//             // Set the appropriate expiration based on the expires_in field
//             self.token_expiration = Some(Utc::now() + Duration::seconds(response_body.expires_in as i64));

//             Ok(())
//         } else {
//             // Handle non-successful responses
//             Err(TwitterAPIError("Failed to refresh token".to_string()).into() )
//         } 

//     }

//     async fn query(&self, endpoint: &str) -> ResultE<reqwest::Response> {
//         let client = Client::new();
//         let url = format!("{}{}", BASE_URL, endpoint);

//         client
//             .get(&url)
//             .header("Authorization", format!("Bearer {}", self.bearer_token.as_ref().ok_or_else(|| TwitterAPIError("Missing token".to_string()))?))
//             .send()
//             .await
//             .map_err(|e| TwitterAPIError(format!("Failed to send request: {}", e)).into())
//     }
// }

// #[async_trait]
// impl ExternalData for TwitterAPI {
//     async fn search(
//         &mut self,
//         keywords: Vec<String>,
//         page_token: Option<String>,
//         last24h: bool,
//     ) -> ResultE<(Vec<Url>, Option<String>)> {
//         self.ensure_token_valid().await?;

//         // Construct the search endpoint
//         let mut endpoint = format!(
//             "search/tweets?query={}&max_results={}",
//             keywords.join(" OR "),
//             PAGE_SIZE
//         );


//         // Optionally add parameters like 'page_token' and 'last24h' to the endpoint if needed
//         if let Some(token) = page_token {
//             endpoint.push_str(&format!("&pagination_token={}", token));
//         }

//         if last24h {
//             // Implement date range filtering based on Twitter API documentation
//             // ...
//         }

//         // Make the API request
//         let response = self.query(&endpoint).await?;

//         if response.status().is_success() {
//             // Parse the JSON response
//             let response_body: Value = response.json().await?;

//             // Extract URLs of tweets from the response JSON
//             let mut tweet_urls = Vec::new();
//             // Implement tweet URL extraction logic based on Twitter API response
//             // ...

//             let next_page_token = None; // Implement pagination token extraction

//             Ok((tweet_urls, next_page_token))
//         } else {
//             Err("Failed to fetch search results".into())
//         }
//     }

//     async fn search_by_category(
//         &mut self,
//         word_keys: Vec<String>,
//         page_token: Option<String>,
//         last24h: bool,
//     ) -> ResultE<(Vec<Url>, Option<String>)> {
//         self.ensure_token_valid().await?;
//         Ok((Vec::new(), None))
//     }

//     // Implement other required methods
// }
