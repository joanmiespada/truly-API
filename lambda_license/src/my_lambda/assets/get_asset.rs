use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::config::Config;
use lib_licenses::errors::block_tx::{BlockchainTxError, BlockchainTxNoExistsError};
use lib_licenses::services::owners::OwnerService;
use lib_licenses::{
    errors::asset::{AssetDynamoDBError, AssetNoExistsError},
    models::{asset::Asset, tx::BlockchainTx},
    services::{
        assets::{AssetManipulation, AssetService},
        block_tx::{BlockchainTxManipulation, BlockchainTxService},
    },
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::instrument;
use uuid::Uuid;
use validator::ValidationError;

use crate::my_lambda::{build_resp, build_resp_env};

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
                    config.env_vars().environment(),
                    e,
                    StatusCode::INTERNAL_SERVER_ERROR,
                );
            }
        }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AssetTx {
    pub asset: Asset,
    pub tx: Option<BlockchainTx>,
}
#[instrument]
pub async fn get_asset_by_shorter(
    req: &Request,
    _c: &Context,
    config: &Config,
    asset_service: &AssetService,
    tx_service: &BlockchainTxService,
    shorter_id: &String,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let op_res = asset_service.get_by_shorter(shorter_id).await;
    match op_res {
        Ok(asset) => match asset.minted_tx() {
            None => {
                let res = AssetTx {
                    asset: asset.to_owned(),
                    tx: None,
                };
                build_resp(json!(res.to_owned()).to_string(), StatusCode::OK)
            }
            Some(hash) => {
                let tx_op = tx_service.get_by_hash(hash).await;
                match tx_op {
                    Ok(tx) => {
                        let res = AssetTx {
                            asset,
                            tx: Some(tx),
                        };
                        build_resp(json!(res).to_string(), StatusCode::OK)
                    }
                    Err(e) => {
                        if let Some(e) = e.downcast_ref::<BlockchainTxError>() {
                            return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE);
                        } else if let Some(m) = e.downcast_ref::<BlockchainTxNoExistsError>() {
                            return build_resp(m.to_string(), StatusCode::NO_CONTENT);
                        } else {
                            return build_resp_env(
                                config.env_vars().environment(),
                                e,
                                StatusCode::INTERNAL_SERVER_ERROR,
                            );
                        }
                    }
                }
            }
        },
        Err(e) => {
            if let Some(e) = e.downcast_ref::<AssetDynamoDBError>() {
                return build_resp(e.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<AssetNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else if let Some(m) = e.downcast_ref::<ValidationError>() {
                return build_resp(m.to_string(), StatusCode::BAD_REQUEST);
            } else {
                return build_resp_env(
                    config.env_vars().environment(),
                    e,
                    StatusCode::INTERNAL_SERVER_ERROR,
                );
            }
        }
    }
}
