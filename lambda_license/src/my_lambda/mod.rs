mod assets;
pub mod error;
mod nft;
mod video;
mod licenses;

use std::str::FromStr;

use crate::my_lambda::assets::get_asset::get_asset_by_shorter;
use crate::my_lambda::licenses::get_licenses::get_licenses;
use crate::my_lambda::licenses::get_my_license::get_my_licenses_all;
use crate::my_lambda::nft::get_tx::{get_tx, get_txs};
use lambda_http::{http::Method, http::StatusCode, IntoResponse, Request, RequestExt, Response};
use lib_blockchain::services::block_tx::BlockchainTxService;
use lib_blockchain::services::nfts::NFTsService;
use lib_config::config::Config;
use lib_config::environment::{DEV_ENV, STAGE_ENV};
use lib_licenses::services::licenses::LicenseService;
use lib_licenses::services::video::VideoService;
use lib_users::services::users::UsersService;
use lib_util_jwt::{get_header_jwt, JWTSecurityError};
use tracing::info;

use self::assets::create_my_asset::create_my_asset;
use self::assets::get_asset::get_asset;
use self::assets::get_my_asset::get_my_assets_all;
use self::error::ApiLambdaError;
use self::nft::async_create_my_nft::async_create_my_nft_sns;
use self::video::async_create_my_shorter::async_create_my_shorter_sns;
use lib_licenses::services::assets::AssetService;
use lib_licenses::services::owners::OwnerService;
use matchit::Router;
use uuid::Uuid;

fn jwt_mandatory(req: &Request, config: &Config) -> Result<String, Response<String>> {
    match check_jwt_token_as_user_logged(&req, config) {
        Err(e) => Err(build_resp(e.to_string(), StatusCode::UNAUTHORIZED).unwrap()),
        Ok(id) => Ok(id),
    }
}

#[tracing::instrument]
pub async fn function_handler(
    config: &Config,
    asset_service: &AssetService,
    owners_service: &OwnerService,
    blockchain_service: &NFTsService,
    user_service: &UsersService,
    video_service: &VideoService,
    tx_service: &BlockchainTxService,
    license_service: &LicenseService,
    req: Request,
) -> Result<impl IntoResponse, Box<dyn std::error::Error + Send + Sync>> {
    info!("income new request");
    let context = req.lambda_context();
    //let query_string = req.query_string_parameters().to_owned();
    //request.uri().path()
    let user_id;
    // match check_jwt_token_as_user_logged(&req, config) {
    //     Err(e) => {
    //         return build_resp(e.to_string(), StatusCode::UNAUTHORIZED);
    //     }
    //     Ok(id) => user_id = id,
    // }

    let mut router = Router::new();
    router.insert("/api/asset", Some("1"))?;
    router.insert("/api/asset/:id", Some("2"))?;
    router.insert("/api/nft", Some("3"))?;
    router.insert("/api/license", Some("4"))?;
    router.insert("/api/license/:id", Some("44"))?;
    router.insert("/api/shorter/:id", Some("5"))?;
    router.insert("/api/shorter", Some("55"))?;
    router.insert("/api/tx/:hash", Some("6"))?;
    router.insert("/api/txs/:id", Some("7"))?;

    match req.method() {
        &Method::GET => match router.at(req.uri().path()) {
            Err(_) => build_resp(
                "method not allowed".to_string(),
                StatusCode::METHOD_NOT_ALLOWED,
            ),
            Ok(matched) => match matched.value.unwrap() {
                "1" => {
                    match jwt_mandatory(&req, config) {
                        Err(e) => {
                            return Ok(e);
                        }
                        Ok(user) => user_id = user,
                    };
                    get_my_assets_all(
                        &req,
                        &context,
                        config,
                        asset_service,
                        owners_service,
                        &user_id,
                    )
                    .await
                }
                "2" => {
                    // public, not required jwt token
                    let id = matched.params.get("id").unwrap().to_string();
                    let asset_id = Uuid::from_str(id.as_str())?;
                    return get_asset(
                        &req,
                        &context,
                        config,
                        asset_service,
                        owners_service,
                        &asset_id,
                    )
                    .await;
                }
                "4" => {
                    match jwt_mandatory(&req, config) {
                        Err(e) => {
                            return Ok(e);
                        }
                        Ok(user) => user_id = user,
                    };
                    get_my_licenses_all(
                        &req,
                        &context,
                        config,
                        license_service,
                        asset_service,
                        &user_id,
                    )
                    .await
                }
                "44" => {
                    // public, not required jwt token
                    let id = matched.params.get("id").unwrap().to_string();
                    let asset_id = Uuid::from_str(id.as_str())?;
                    return get_licenses(
                        &req,
                        &context,
                        config,
                        asset_service,
                        license_service,
                        &asset_id,
                    )
                    .await;
                }
                "5" => {
                    // public, not required jwt token
                    let shorter_id = matched.params.get("id").unwrap().to_string();
                    return get_asset_by_shorter(
                        &req,
                        &context,
                        config,
                        asset_service,
                        tx_service,
                        &shorter_id,
                    )
                    .await;
                }
                "6" => {
                    match jwt_mandatory(&req, config) {
                        Err(e) => {
                            return Ok(e);
                        }
                        Ok(_) => {}
                    };
                    let tx_hash = matched.params.get("hash").unwrap().to_string();
                    return get_tx(&req, &context, config, tx_service, &tx_hash).await;
                }
                "7" => {
                    match jwt_mandatory(&req, config) {
                        Err(e) => {
                            return Ok(e);
                        }
                        Ok(_) => {}
                    };
                    let id = matched.params.get("id").unwrap().to_string();
                    let asset_id = Uuid::from_str(id.as_str())?;
                    return get_txs(&req, &context, config, tx_service, &asset_id).await;
                }
                _ => build_resp(
                    "method not allowed".to_string(),
                    StatusCode::METHOD_NOT_ALLOWED,
                ),
            },
        },
        &Method::POST => match router.at(req.uri().path()) {
            Err(_) => build_resp(
                "method not allowed".to_string(),
                StatusCode::METHOD_NOT_ALLOWED,
            ),
            Ok(matched) => match matched.value.unwrap() {
                "1" => {
                    match jwt_mandatory(&req, config) {
                        Err(e) => {
                            return Ok(e);
                        }
                        Ok(user) => user_id = user,
                    };
                    create_my_asset(
                        &req,
                        &context,
                        config,
                        asset_service,
                        owners_service,
                        &user_id,
                    )
                    .await
                }
                // only useful in local development
                // "4" => {
                //     //let id = matched.params.get("id").unwrap().to_string();
                //     //let asset_id = Uuid::from_str(id.as_str())?;

                //     match jwt_mandatory(&req, config){
                //         Err(e) => { return Ok(e);},
                //         Ok(user)=> user_id = user
                //     };

                //     return create_my_nft(
                //         &req,
                //         &context,
                //         config,
                //         asset_service,
                //         owners_service,
                //         blockchain_service,
                //         user_service,
                //         &user_id,
                //     )
                //     .await;
                // }
                "3" => {
                    //let id = matched.params.get("id").unwrap().to_string();
                    //let asset_id = Uuid::from_str(id.as_str())?;

                    match jwt_mandatory(&req, config) {
                        Err(e) => {
                            return Ok(e);
                        }
                        Ok(user) => user_id = user,
                    };

                    return async_create_my_nft_sns(
                        &req,
                        &context,
                        config,
                        asset_service,
                        owners_service,
                        blockchain_service,
                        user_service,
                        &user_id,
                    )
                    .await;
                }

                "55" => {

                    match jwt_mandatory(&req, config) {
                        Err(e) => {
                            return Ok(e);
                        }
                        Ok(user) => user_id = user,
                    };

                    return async_create_my_shorter_sns(
                        &req,
                        &context,
                        config,
                        asset_service,
                        video_service,
                        &user_id,
                    )
                    .await;
                }

                &_ => build_resp(
                    "method not allowed".to_string(),
                    StatusCode::METHOD_NOT_ALLOWED,
                ),
            },
        },
        _ => build_resp(
            "http verb doesn't use it here".to_string(),
            StatusCode::METHOD_NOT_ALLOWED,
        ),
    }
}

fn build_resp(
    msg: String,
    status_code: StatusCode,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let res = Response::builder()
        .status(status_code)
        .header("content-type", "text/json")
        .body(msg);
    match res {
        Err(e) => Err(ApiLambdaError { 0: e.to_string() }.into()),
        Ok(resp) => Ok(resp),
    }
}

fn build_resp_no_cache(
    msg: String,
    status_code: StatusCode,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let res = Response::builder()
        .status(status_code)
        .header("content-type", "text/json")
        .header("cache-control", "no-cache,max-age=0")
        .body(msg);
    match res {
        Err(e) => Err(ApiLambdaError { 0: e.to_string() }.into()),
        Ok(resp) => Ok(resp),
    }
}

fn build_resp_env(
    env: &String,
    error: Box<dyn std::error::Error + Send + Sync>,
    status_code: StatusCode,
) -> Result<Response<String>, Box<dyn std::error::Error + Send + Sync>> {
    let msg: String;
    if env == DEV_ENV || env == STAGE_ENV {
        msg = format!("{}", error);
    } else {
        msg = "".to_string();
    }

    let res = Response::builder()
        .status(status_code)
        .header("content-type", "text/json")
        .header("cache-control", "max-age=300") //5 minutes
        .body(msg);
    match res {
        Err(e) => Err(ApiLambdaError { 0: e.to_string() }.into()),
        Ok(resp) => Ok(resp),
    }
}
fn check_jwt_token_as_user_logged(
    req: &Request,
    config: &Config,
) -> Result<String, JWTSecurityError> {
    let user_id;
    let req_headers = req.headers();

    let jwt_secret = config.env_vars().jwt_token_base();
    let claim_ops = get_header_jwt(req_headers, jwt_secret);

    match claim_ops {
        Ok(clm) => {
            user_id = clm.uid;
        }
        Err(e) => {
            return Err(e);
        }
    }
    Ok(user_id)
}
