use lambda_http::RequestExt;
use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::config::Config;
use lib_licenses::services::assets::AssetService;
use lib_licenses::services::video::{VideoService, VideoManipulation};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use validator::Validate;

use crate::my_lambda::build_resp;

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct CreateShorterAsync {
    pub asset_id: Uuid,
}

#[tracing::instrument]
pub async fn async_create_my_shorter_sns(
    req: &Request,
    _c: &Context,
    config: &Config,
    asset_service: &AssetService,
    video_service: &VideoService,
    user_id: &String,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let new_shorter_video;
    match req.payload::<CreateShorterAsync>() {
        Err(e) => {
            return build_resp(e.to_string(), StatusCode::BAD_REQUEST);
        }
        Ok(op_payload) => match op_payload {
            None => {
                return build_resp("no payload found".to_string(), StatusCode::BAD_REQUEST);
            }
            Some(payload) => match payload.validate() {
                Err(e) => {
                    return build_resp(e.to_string(), StatusCode::BAD_REQUEST);
                }
                Ok(_) => {
                    new_shorter_video = payload.clone();
                }
            },
        },
    }

    let post_request_op = video_service.shorter_video_async(&new_shorter_video.asset_id, &user_id).await;

    //check user owns the asset.
    //let checks_op = asset_service.get_by_user_asset_id( &new_shorter_video.asset_id, &user_id).await;

    //let asset: Asset;
    match post_request_op {
        Err(e)=>{
                return build_resp(e.to_string(), StatusCode::CONFLICT);
        },
        Ok(val)=>{

            return build_resp(val, StatusCode::OK);
            /* 
            match *ass.video_licensing_status(){ 
                 VideoLicensingStatus::Scheduled =>  
                    return build_resp("it has been already scheduled. Please await until current process report any new status.".to_string(), StatusCode::CONFLICT),
                VideoLicensingStatus::AlreadyLicensed =>
                    return build_resp("already shorter applied, you can't overwrite it".to_string(), StatusCode::CONFLICT ),
                VideoLicensingStatus::Started =>
                    return build_resp("process started some seconds ago, please await...".to_string(), StatusCode::CONFLICT ),
                
                _ => asset = ass,
            }
            */
        }
    }
/* 
    let new_shorter = CreateShorter {
        keep_original:true,
        url_file: asset.url().clone().unwrap() ,
        hash: asset.hash().clone().unwrap(),
        asset_id: new_shorter_video.asset_id,
        user_id: user_id.clone() 
    };

    let json_text = serde_json::to_string(&new_shorter)?;

    let message = SNSMessage {
        body: json_text.to_owned(),
    };

    let topic_arn = config.env_vars().topic_arn_shorter_video_start().to_owned();

    let enqueded_op = send_sns(config, &message, topic_arn).await;

    match enqueded_op {
        Err(e) => {
            if let Some(m) = e.downcast_ref::<AsyncOpError>() {
                return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else {
                return build_resp(
                    "unknonw error working with video licensing by shorter".to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                );
            }
        }
        Ok(val) => {
            asset_service
                .shorter_video_status (&new_shorter_video.asset_id, &Some("".to_string()), VideoLicensingStatus::Scheduled)
                .await?;
            return build_resp(val, StatusCode::OK);
        }
    };
    */
}



