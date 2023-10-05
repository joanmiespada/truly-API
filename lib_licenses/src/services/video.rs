use async_trait::async_trait;
use lib_async_ops::{
    errors::AsyncOpError,
    sns::{send, SNSMessage},
};
use lib_config::config::Config;
use lib_video_objs::shorter::CreateShorter;
use uuid::Uuid;
use tracing::info;

use crate::{
    errors::video::VideoError,
    models::{asset::{Asset, VideoLicensingStatus}, hash::CreateHashes, video::{MatchAPIResponse, SimilarResponse, SimilarItem}},
};

use super::assets::{AssetManipulation, AssetService};

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;


#[async_trait]
pub trait VideoManipulation {
    async fn shorter_video_async(&self, asset_id: &Uuid, user_id: &String) -> ResultE<String>;
    async fn compute_hash_and_similarities_async(&self, asset_id: &Uuid) -> ResultE<String>;
    async fn get_similar_hashes(&self, asset_id: &Uuid) -> ResultE<SimilarResponse>;
}

#[derive(Debug)]
pub struct VideoService {
    asset_service: AssetService,
    config: Config,
}

impl VideoService {
    pub fn new(assets: AssetService, conf: Config) -> VideoService {
        VideoService {
            asset_service: assets,
            config: conf,
        }
    }
}

#[async_trait]
impl VideoManipulation for VideoService {
    #[tracing::instrument()]
    async fn shorter_video_async(&self, asset_id: &Uuid, user_id: &String) -> ResultE<String> {
        //check user owns the asset.
        let checks_op = self
            .asset_service
            .get_by_user_asset_id(asset_id, user_id)
            .await;

        let asset: Asset;
        match checks_op {
            Err(e) => {
                return Err(VideoError {
                    0: format!(
                        "user {} doesn't own asset {} error: {:?}",
                        user_id, asset_id, e
                    ),
                }
                .into());
            }
            Ok(ass) => {
                match *ass.video_licensing_status() {
                    VideoLicensingStatus::Scheduled => {
                        return Err(VideoError{0: format!("it has been already scheduled. Please await until current process report any new status.")}.into());
                    } //build_resp("it has been already scheduled. Please await until current process report any new status.".to_string(), StatusCode::CONFLICT),
                    VideoLicensingStatus::AlreadyLicensed => {
                        return Err(VideoError {
                            0: format!("already shorter applied, you can't overwrite it"),
                        }
                        .into());
                    } //build_resp("already shorter applied, you can't overwrite it".to_string(), StatusCode::CONFLICT ),
                    VideoLicensingStatus::Started => {
                        return Err(VideoError {
                            0: format!("process started some seconds ago, please await..."),
                        }
                        .into());
                    } //build_resp("process started some seconds ago, please await...".to_string(), StatusCode::CONFLICT ),

                    _ => asset = ass,
                }
            }
        }

        let new_shorter = CreateShorter {
            keep_original: true,
            url_file: asset.url().clone().unwrap(),
            hash: asset.hash().clone().unwrap(),
            hash_algorithm: asset.hash_algorithm().clone().unwrap(),
            asset_id: asset_id.clone(),
            user_id: user_id.clone(),
        };

        let json_text = serde_json::to_string(&new_shorter)?;

        let message = SNSMessage {
            body: json_text.to_owned(),
        };

        let topic_arn = self
            .config
            .env_vars()
            .topic_arn_shorter_video_start()
            .unwrap();

        let enqueded_op = send(&self.config, &message, topic_arn).await;

        match enqueded_op {
            Err(e) => {
                if let Some(m) = e.downcast_ref::<AsyncOpError>() {
                    return Err(VideoError {
                        0: format!("{:?}", m),
                    }
                    .into()); //build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
                } else {
                    return Err(VideoError {
                        0: format!("unknown error with video licensing at sns topic"),
                    }
                    .into()); //build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
                              //return build_resp(
                              //    "unknonw error working with video licensing by shorter".to_string(),
                              //    StatusCode::INTERNAL_SERVER_ERROR,
                              //);
                }
            }
            Ok(val) => {
                self.asset_service
                    .shorter_video_status(
                        &asset_id,
                        &Some("".to_string()), //clean previous messages
                        VideoLicensingStatus::Scheduled, //new status
                    )
                    .await?;
                //return build_resp(val, StatusCode::OK);
                return Ok(val);
            }
        };
    }

    #[tracing::instrument()]
    async fn compute_hash_and_similarities_async(&self, asset_id: &Uuid) -> ResultE<String>{
        //check user owns the asset.
        let checks_op = self
            .asset_service
            .get_by_id(asset_id)
            .await;
        if let Err(e)= checks_op {
                return Err(VideoError {
                    0: format!(
                        "asset {} doesn't exist error: {:?}",
                        asset_id, e
                    ),
                }
                .into());
        }
        let asset = checks_op.unwrap();
        let new_hashes = CreateHashes {
            source_url: asset.url().clone().unwrap(),
            asset_id: asset_id.clone(),
        };
        
        let json_text = serde_json::to_string(&new_hashes)?;

        let message = SNSMessage {
            body: json_text.to_owned(),
        };

        let topic_arn = self
            .config
            .env_vars()
            .topic_arn_hashes_similars_start()
            .unwrap();

        let enqueded_op = send(&self.config, &message, topic_arn).await;

        match enqueded_op {
            Err(e) => {
                if let Some(m) = e.downcast_ref::<AsyncOpError>() {
                    return Err(VideoError {
                        0: format!("{:?}", m),
                    }
                    .into());
                } else {
                    return Err(VideoError {
                        0: format!("unknown error with video hashes computation at sns topic"),
                    }
                    .into());
                }
            }
            Ok(val) => {
                return Ok(val);
            }
        };

    }

    //#[tracing::instrument()]
    async fn get_similar_hashes(&self, asset_id: &Uuid) -> ResultE<SimilarResponse>{
        let url_matchapi:String = format!("{}?asset_id={}", self.config.env_vars().matchapi_endpoint().unwrap().to_string(), asset_id.to_string());
        info!("calling external service: {}", url_matchapi);

        let mut resp: MatchAPIResponse = reqwest::get(url_matchapi).await?.json().await?;

        let mut similar_items :Vec<SimilarItem>= vec![];

        for item in &mut resp.similars {
            let ass1 = self.asset_service.get_by_id(&item.asset_id).await?;
            let asset_url = ass1.url().clone();

            let frame_second;
            match item.frame_second.parse::<f32>(){
                Ok(value) => frame_second=Some(value),
                Err(_) => frame_second= None
            };

            let aux =SimilarItem{
                asset_id: item.asset_id.clone(),
                frame_id: item.frame_id.clone(),
                frame_url: item.frame_url.clone(),
                frame_second,
                asset_url
            };

            similar_items.push(aux);
        };

        let result = SimilarResponse{
            next_token: resp.next_token,
            similars: similar_items
        };

        Ok(result)
    }

}
