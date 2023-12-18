use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use derive_builder::Builder;
use lib_config::{config::Config, result::ResultE};
use reqwest::{Client, Url};
use reqwest::{header, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_urlencoded;
//use url::Url;

use crate::ExternalData;

use self::error::TwitchAPIError;

pub mod error;

const MAX_RESULTS: i32 = 50;
const BASE_URL: &str = "https://api.twitch.tv/helix/";

#[derive(Serialize)]
struct OAuthRequest {
    client_id: String,
    client_secret: String,
    grant_type: String,
}

#[derive(Deserialize)]
struct OAuthResponse {
    access_token: String,
    expires_in: u64, // Duration in seconds
                     // Include other fields as needed
}

#[derive(Clone, Debug, Builder)]
pub struct TwitchAPI {
    client_id: String,
    client_secret: String,
    token: Option<String>,
    token_expiration: Option<DateTime<Utc>>,
}

impl TwitchAPI {
    pub fn new(conf: &Config) -> TwitchAPI {
        TwitchAPI {
            client_id: conf.env_vars().twitch_client_id().clone().unwrap(),
            client_secret: conf.env_vars().twitch_client_secret().clone().unwrap(),
            token: None,
            token_expiration: None,
        }
    }

    async fn get_oauth_token(&mut self) -> ResultE<()> {
        let request_body = OAuthRequest {
            client_id: self.client_id.to_owned(),
            client_secret: self.client_secret.to_owned(),
            grant_type: "client_credentials".to_owned(),
        };

        let client = Client::new();
        let res = client
            .post("https://id.twitch.tv/oauth2/token")
            .form(&request_body)
            .send()
            .await
            .map_err(|e| TwitchAPIError(format!("Failed to send request: {}", e)))?;

        match res.status() {
            StatusCode::OK => {
                let response: OAuthResponse = res
                    .json::<OAuthResponse>()
                    .await
                    .map_err(|e| TwitchAPIError(format!("Failed to parse response: {}", e)))?;

                self.token = Some(response.access_token);
                self.token_expiration =
                    Some(Utc::now() + Duration::seconds(response.expires_in as i64));
                Ok(())
            }
            _ => Err(TwitchAPIError("Failed to retrieve token".to_string()).into()),
        }
    }

    async fn ensure_token_valid(&mut self) -> ResultE<()> {
        if let Some(expiration) = self.token_expiration {
            if Utc::now() >= expiration {
                self.get_oauth_token().await?;
            }
        } else {
            self.get_oauth_token().await?;
        }
        Ok(())
    }

    async fn query(&self, url: &str) -> Result<reqwest::Response, TwitchAPIError> {
        let client = Client::new();
        client
            .get(url)
            .header(
                header::AUTHORIZATION,
                format!(
                    "Bearer {}",
                    self.token
                        .as_ref()
                        .ok_or_else(|| TwitchAPIError("Missing token".to_string()))?
                ),
            )
            .header("Client-Id", &self.client_id)
            .send()
            .await
            .map_err(|e| TwitchAPIError(e.to_string()))
    }

    async fn get_category_id(&self, category_label: &String) -> ResultE<Option<String>> {
        let url = format!("{}search/categories?query={}", BASE_URL, category_label);
        let response = self.query(&url).await?;
        match response.status() {
            StatusCode::OK => {
                let response_text = response.text().await.unwrap();
                let json: Value = serde_json::from_str(&response_text).unwrap();
                if let Some(datas) = json["data"].as_array() {
                    if datas.len() == 0 {
                        Ok(None)
                    } else {
                        if let Some(first) = datas[0]["id"].as_str() {
                            Ok(Some(first.to_string()))
                        } else {
                            Ok(None)
                        }
                    }
                } else {
                    Ok(None)
                }
            }
            StatusCode::UNAUTHORIZED => Err(TwitchAPIError("login again".to_string()).into()),
            StatusCode::TOO_MANY_REQUESTS => {
                Err(TwitchAPIError("Too many requests".to_string()).into())
            }
            _ => Err(TwitchAPIError("Unknown error".to_string()).into()),
        }
    }
}

#[async_trait]
impl ExternalData for TwitchAPI {
    async fn search(
        &mut self,
        word_keys: Vec<String>,
        page_token: Option<String>,
        _: bool,
    ) -> ResultE<(Vec<Url>, Option<String>)> {
        self.ensure_token_valid().await?;

        let word_keys2: Vec<String> = word_keys
            .iter()
            .map(|item| item.replace(" ", "%20"))
            .collect();
        let words = word_keys2.join("%20"); // not sure if it's correct

        let mut url = format!(
            "{}search/channels?query={}&first={}",
            BASE_URL, words, MAX_RESULTS
        );
        //println!("{}",url);

        if let Some(page_token) = page_token {
            url.push_str(&format!("&after={}", page_token));
        }

        let response = self.query(&url).await?;

        match response.status() {
            StatusCode::OK => {
                let response_text = response.text().await.unwrap();
                let json: Value = serde_json::from_str(&response_text).unwrap();

                let next_page_token = json["pagination"]["cursor"].as_str().map(String::from);
                let mut result = Vec::new();

                if let Some(channels) = json["data"].as_array() {
                    for channel in channels {
                        if let Some(idd) = channel["display_name"].as_str() {
                            let video_url_1 = Url::parse(&format!("https://twitch.tv/{}", idd))?;

                            result.push(video_url_1.to_owned());
                            println!("{}", video_url_1);
                        }
                    }
                }

                Ok((result, next_page_token))
            }
            StatusCode::UNAUTHORIZED => Err(TwitchAPIError("login again".to_string()).into()),
            StatusCode::FORBIDDEN => Err(TwitchAPIError("Forbidden".to_string()).into()),
            StatusCode::TOO_MANY_REQUESTS => {
                Err(TwitchAPIError("Too many requests".to_string()).into())
            }
            _ => Err(TwitchAPIError("Unknown error".to_string()).into()),
        }
    }

    async fn search_by_category(
        &mut self,
        game_name: Vec<String>,
        page_token: Option<String>,
        last24h: bool,
    ) -> ResultE<(Vec<Url>, Option<String>)> {
        self.ensure_token_valid().await?;

        let game_id = self.get_category_id(&game_name[0]).await?;
        let mut result = Vec::new();

        if game_id == None {
            return Ok((Vec::new(), None));
        }

        let mut url = format!(
            "{}clips?game_id={}&first={}",
            BASE_URL,
            game_id.unwrap(),
            MAX_RESULTS
        );

        if let Some(page_token) = page_token {
            url.push_str(&format!("&after={}", page_token));
        }

        if last24h {
            let nw: DateTime<Utc> = Utc::now();
            let now = nw.to_rfc3339();
            let finish = (nw + Duration::hours(24)).to_rfc3339();

            let params = [("started_at", &now), ("ended_at", &finish)];

            let encoded_params = serde_urlencoded::to_string(params)?;

            url.push_str(&format!("&{}", encoded_params ));
        }

        //println!("{}",url);
        let response1 = self.query(&url).await?;

        //println!( "status: {}",response1.status() );

        match response1.status() {
            StatusCode::OK => {
                let response_text1 = response1.text().await.unwrap();
                let json1: Value = serde_json::from_str(&response_text1).unwrap();

                let next_page_token = json1["pagination"]["cursor"].as_str().map(String::from);

                if let Some(clips) = json1["data"].as_array() {
                    for clip in clips {
                        if let Some(idd) = clip["url"].as_str() {
                            //println!("{}",idd);
                            let video_url_1 = Url::parse(&format!("{}", idd))?;
                            result.push( video_url_1);
                        }
                    }
                }

                Ok((result, next_page_token))
            }
            StatusCode::UNAUTHORIZED => Err(TwitchAPIError("login again".to_string()).into()),
            StatusCode::FORBIDDEN => Err(TwitchAPIError("Forbidden".to_string()).into()),
            StatusCode::TOO_MANY_REQUESTS => {
                Err(TwitchAPIError("Too many requests".to_string()).into())
            }
            _ => Err(TwitchAPIError( "Unknown error:".to_string()).into()),
        }
    }
}
