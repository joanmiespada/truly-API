use chrono::{DateTime, Utc};
use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
//use lib_blockchain::errors::block_tx::{BlockchainTxError, BlockchainTxNoExistsError};
//use lib_blockchain::models::block_tx::BlockchainTx;
//use lib_blockchain::services::block_tx::{BlockchainTxManipulation, BlockchainTxService};
use lib_config::config::Config;
use lib_licenses::models::asset::{AssetStatus, VideoLicensingStatus};
use lib_licenses::models::license::License;
use lib_licenses::services::licenses::{LicenseManipulation, LicenseService};
use lib_licenses::services::owners::OwnerService;
use lib_licenses::{
    errors::asset::{AssetDynamoDBError, AssetNoExistsError},
    models::asset::Asset,
    services::assets::{AssetManipulation, AssetService},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::instrument;
use uuid::Uuid;
use validator::ValidationError;

use crate::my_lambda::{build_resp, build_resp_env, build_resp_no_cache};

#[instrument]
pub async fn get_asset(
    req: &Request,
    _c: &Context,
    config: &Config,
    asset_service: &AssetService,
    owner_service: &OwnerService,
    asset_id: &Uuid,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let op_res = asset_service.get_by_id_enhanced(asset_id).await;
    match op_res {
        Ok(asset) => build_resp(json!(asset).to_string(), StatusCode::OK),
        Err(e) => {
            if let Some(e) = e.downcast_ref::<AssetDynamoDBError>() {
                return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<AssetNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else if let Some(m) = e.downcast_ref::<ValidationError>() {
                return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
            } else {
                return build_resp_env(
                    &config.env_vars().environment().unwrap(),
                    e,
                    StatusCode::INTERNAL_SERVER_ERROR,
                );
            }
        }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AssetTx {
    pub id: Uuid,
    pub creation_time: DateTime<Utc>,
    pub last_update_time: DateTime<Utc>,
    pub status: AssetStatus,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub hash: String,
    pub license: Option<Vec<License>>,
    //pub mint_status: MintingStatus,
    pub video_licensing_status: VideoLicensingStatus,
    //pub tx: Option<BlockchainTx>,
}

impl AssetTx {
    pub fn new(asset: &Asset, //tx: &Option<BlockchainTx>, 
                lics: &Option<Vec<License>>) -> AssetTx {
        AssetTx {
            id: asset.id().clone(),
            creation_time: asset.creation_time().clone(),
            last_update_time: asset.last_update_time().clone(),
            status: asset.state().clone(),
            latitude: asset.latitude().clone(),
            longitude: asset.longitude().clone(),
            hash: asset.hash().clone().unwrap().clone(),
            //mint_status: asset.mint_status().clone(),
            video_licensing_status: asset.video_licensing_status().clone(),
            //tx: tx.clone(),
            license: lics.clone(),
        }
    }
}

#[instrument]
pub async fn get_asset_by_shorter(
    req: &Request,
    _c: &Context,
    config: &Config,
    asset_service: &AssetService,
    //tx_service: &BlockchainTxService,
    license_service: &LicenseService,
    shorter_id: &String,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let op_res = asset_service.get_by_shorter(shorter_id).await;
    match op_res {
        Ok(asset) => {
            let licenses = license_service.get_by_asset(asset.id()).await?;
            //match asset.minted_tx() {
            //    None => {
                    let res = AssetTx::new(&asset, //&None,
                                             &Some(licenses));

                    build_resp_no_cache(json!(res.to_owned()).to_string(), StatusCode::OK)
            //    }

            /*     Some(hash) => {
                    let tx_op = tx_service.get_by_id(hash).await;
                    match tx_op {
                        Ok(tx) => {
                            let res = AssetTx::new(&asset, &Some(tx), &Some(licenses));
                            build_resp_no_cache(json!(res).to_string(), StatusCode::OK)
                        }
                        Err(e) => {
                            if let Some(e) = e.downcast_ref::<BlockchainTxError>() {
                                return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE);
                            } else if let Some(m) = e.downcast_ref::<BlockchainTxNoExistsError>() {
                                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
                            } else {
                                return build_resp_env(
                                    &config.env_vars().environment().unwrap(),
                                    e,
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                );
                            }
                        }
                    }
                }
            }*/
        }
        Err(e) => {
            if let Some(e) = e.downcast_ref::<AssetDynamoDBError>() {
                return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<AssetNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else if let Some(m) = e.downcast_ref::<ValidationError>() {
                return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
            } else {
                return build_resp_env(
                    &config.env_vars().environment().unwrap(),
                    e,
                    StatusCode::INTERNAL_SERVER_ERROR,
                );
            }
        }
    }
}
