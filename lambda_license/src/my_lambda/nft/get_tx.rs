use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::config::Config;
use lib_licenses::errors::block_tx::{BlockchainTxError, BlockchainTxNoExistsError};
use lib_licenses::services::block_tx::{BlockchainTxManipulation, BlockchainTxService};
use serde_json::json;
use uuid::Uuid;

use crate::my_lambda::build_resp;

#[tracing::instrument]
pub async fn get_tx(
    req: &Request,
    _c: &Context,
    config: &Config,
    tx_service: &BlockchainTxService,
    tx_hash: &String,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let op_res = tx_service.get_by_id(tx_hash).await;

    let transaction = match op_res {
        Err(e) => {
            if let Some(m) = e.downcast_ref::<BlockchainTxError>() {
                return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<BlockchainTxNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else {
                return build_resp(
                    "unknonw error retrieving Tx".to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                );
            }
        }
        Ok(tx) => format!("{}", tx),
    };

    return build_resp(transaction, StatusCode::OK);
}

#[tracing::instrument]
pub async fn get_txs(
    req: &Request,
    _c: &Context,
    config: &Config,
    tx_service: &BlockchainTxService,
    asset_id: &Uuid,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let op_res = tx_service.get_by_asset_id(asset_id).await;

    match op_res {
        Err(e) => {
            if let Some(m) = e.downcast_ref::<BlockchainTxError>() {
                return build_resp(m.to_string(), StatusCode::SERVICE_UNAVAILABLE);
            } else if let Some(m) = e.downcast_ref::<BlockchainTxNoExistsError>() {
                return build_resp(m.to_string(), StatusCode::NO_CONTENT);
            } else {
                return build_resp(
                    "unknonw error retrieving Tx".to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                );
            }
        }
        Ok(tx) => {
            return build_resp(json!(tx).to_string(), StatusCode::OK);
        }
    };
}
