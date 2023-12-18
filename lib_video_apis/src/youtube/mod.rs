use std::{fs::File, io::Write};

use lib_config::{environment::EnvironmentVariables, config::Config, result::ResultE};
use reqwest::{StatusCode, header};
use serde_json::Value;
use url::Url;
use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use derive_builder::Builder;

use crate::ExternalData;

use self::error::YoutubeAPIError;
pub mod error;



const MAX_RESULTS:i32=50;
const BASE_DOMAIN:&str = "https://youtube.googleapis.com/youtube/v3/";

#[derive(Clone, Debug, Builder)]
pub struct YoutubeAPI {
    environment_vars: EnvironmentVariables,
}

impl YoutubeAPI {
    pub fn new(conf: &Config) -> YoutubeAPI {
        YoutubeAPI {
            environment_vars: conf.env_vars().clone(),
        }
    }
}

#[async_trait]
impl ExternalData for YoutubeAPI{
    async fn search(&mut self, word_keys: Vec<String>, page_token: Option<String>, last24h:bool) -> ResultE<(Vec<Url>, Option<String>)>{

        let mut result = Vec::new();

        let api_key = self.environment_vars.youtube_api_key().unwrap();

        let words = word_keys.join("+");

        let mut url = format!(
            //"{}search?q={}&key={}&part=snippet&order=date&maxResults={}&type=video",
            "{}search?q={}&key={}&part=snippet&order=date&maxResults={}&type=video&videoCaption=any",
            BASE_DOMAIN,words, api_key, MAX_RESULTS
        );
        //println!("{}",url);

        if let Some(token) = page_token {
            url.push_str(&format!("&pageToken={}", token));
        }

        let client = reqwest::Client::new();
        let response = client.get(&url)
            .header(header::ACCEPT_ENCODING, "gzip")
            .send()
            .await;

        let response_text;
        match response {
            Ok(resp)=> {
                response_text = resp.text().await.unwrap();
                //println!("{}",response_text)
                //let mut file = File::create("./deleteme.json")?;
                //file.write_all(response_text.as_bytes())?;
            },
            Err(e)=>{
                if e.status() == Some(StatusCode::FORBIDDEN) {
                    return Err(YoutubeAPIError("forbidden".to_string()).into() )
                }else if e.status() == Some(StatusCode::TOO_MANY_REQUESTS) {
                    return Err(YoutubeAPIError("too_many_request".to_string()).into() )
                }else {
                    return Err(YoutubeAPIError(e.to_string()).into() )
                }
            }
            
        }

        //let response_text = response.text().await?;
        let json: Value = serde_json::from_str(&response_text).unwrap();

        let next_page_token = json["nextPageToken"].as_str().map(String::from);

        if let Some(items) = json["items"].as_array() {
            for item in items {
                if let Some(video_id) = item["id"]["videoId"].as_str() {
                    let video_url = Url::parse( &format!("https://www.youtube.com/watch?v={}", video_id))?;
                    //println!("{}", video_url);
                    if last24h {

                        if let Some(published_at) = item["snippet"]["publishedAt"].as_str() {
                            let published_date: DateTime<Utc> = published_at.parse()?;
                            let duration_since_published = Utc::now() - published_date;
        
                            if duration_since_published < Duration::days(1) {
                                result.push(video_url);
                            }
                        }

                    }else{
                        result.push(video_url)
                    }
                }
            }
        }

        Ok((result, next_page_token))

    }

    async fn search_by_category(&mut self, word_keys: Vec<String>, page_token: Option<String>, last24h:bool) -> ResultE<(Vec<Url>, Option<String>)>{

        let mut result = Vec::new();

        let api_key = self.environment_vars.youtube_api_key().unwrap();

        let words = word_keys.join(",");

        let mut url = format!(
            "{}videos?videoCategoryId={}&key={}&part=snippet&order=date&maxResults={}&chart=mostPopular",
            BASE_DOMAIN,words, api_key, MAX_RESULTS
        );
        //println!("{}",url);

        if let Some(token) = page_token {
            url.push_str(&format!("&pageToken={}", token));
        }

        let client = reqwest::Client::new();
        let response = client.get(&url)
            .header(header::ACCEPT_ENCODING, "gzip")
            .send()
            .await;

        let response_text;
        match response {
            Ok(resp)=> {
                response_text = resp.text().await.unwrap();
                //println!("{}",response_text)
                let mut file = File::create("./deleteme-category.json")?;
                file.write_all(response_text.as_bytes())?;
            },
            Err(e)=>{
                if e.status() == Some(StatusCode::FORBIDDEN) {
                    return Err(YoutubeAPIError("forbidden".to_string()).into() )
                }else if e.status() == Some(StatusCode::TOO_MANY_REQUESTS) {
                    return Err(YoutubeAPIError("too_many_request".to_string()).into() )
                }else {
                    return Err(YoutubeAPIError(e.to_string()).into() )
                }
            }
            
        }

        //let response_text = response.text().await?;
        let json: Value = serde_json::from_str(&response_text).unwrap();

        let next_page_token = json["nextPageToken"].as_str().map(String::from);

        if let Some(items) = json["items"].as_array() {
            for item in items {
                if let Some(video_id) = item["id"].as_str() {
                    let video_url = Url::parse( &format!("https://www.youtube.com/watch?v={}", video_id))?;
                    //println!("{}", video_url);
                    if last24h {

                        if let Some(published_at) = item["snippet"]["publishedAt"].as_str() {
                            let published_date: DateTime<Utc> = published_at.parse()?;
                            let duration_since_published = Utc::now() - published_date;
        
                            if duration_since_published < Duration::days(1) {
                                result.push(video_url);
                            }
                        }

                    }else{
                        result.push(video_url)
                    }
                }
            }
        }

        Ok((result, next_page_token))

    }

}
