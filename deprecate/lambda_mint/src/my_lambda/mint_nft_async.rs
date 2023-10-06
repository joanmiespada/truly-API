use lib_async_ops::sns::{send as send_sns, SNSMessage};
use lib_blockchain::services::nfts::{CreateNFTAsync, NFTsManipulation, NFTsService};
use lib_config::config::Config;
use lib_licenses::{
    models::asset::MintingStatus,
    services::assets::{AssetManipulation, AssetService},
};
use tracing::{error, info, instrument};

const MAX_RETRIES: usize = 5;

//#[instrument]
pub async fn async_minting(
    data: &mut CreateNFTAsync,
    config: &Config,
    blockchain_service: &NFTsService,
    asset_service: &AssetService,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if data.get_tries() > MAX_RETRIES {
        //dicard minting
        info!("minting operation discarded! too many retries");
        info!("{}", data);

        let asset_op = asset_service.get_by_id(&data.asset_id).await;
        match asset_op {
            Err(e) => error!("{}", e),
            Ok(ass) => {
                asset_service
                    .mint_status(ass.id(), &None, MintingStatus::Error)
                    .await?;
            }
        }

        let json_text = serde_json::to_string(&data)?;

        let message = SNSMessage {
            body: json_text.to_owned(),
        };
        let topic_arn = config.env_vars().topic_arn_mint_fails().unwrap();
        info!("sending message to discarded mint at topic: {}", topic_arn);
        let op_sent = send_sns(config, &message, topic_arn.to_owned()).await;
        match op_sent {
            Ok(_) => {
                info!("message notified at topic {}", topic_arn.to_owned());
            }
            Err(e) => {
                error!(
                    "message failed when posting in the topic {}",
                    topic_arn.to_owned()
                );
                error!("{}", data);
                error!("{}", e);
            }
        }
    } else {
        info!("minting...");
        info!("{}", data);
        let op_res = blockchain_service
            .try_mint(&data.asset_id, &data.user_id, &Some(data.price))
            .await;

        match op_res {
            Err(e) => {
                let err_str = e.to_string();
                info!("{}", err_str);
                if err_str.contains("token is already in use") {
                    info!("pervios tries were successfully minted, let's discard this one");
                } else {
                    error!("failed when minting, retring operation for later");
                    error!("{}", e.to_string());
                    //re-scheduele again
                    data.increase_try();
                    let json_text = serde_json::to_string(&data)?;

                    let message = SNSMessage {
                        body: json_text.to_owned(),
                    };
                    let topic_arn = config.env_vars().topic_arn_mint_async().unwrap();

                    let op_sent = send_sns(config, &message, topic_arn.clone()).await;
                    match op_sent {
                        Ok(_) => {
                            error!("{}", data);
                        }
                        Err(e) => {
                            error!(
                                "message retry failed when posting in the topic {}",
                                topic_arn
                            );
                            error!("{}", data);
                            error!("{}", e);
                        }
                    }
                }
            }
            Ok(tx) => {
                info!("minting successfully!");
                info!("{}", data);
                info!("{}", tx);
            }
        };
    }
    Ok(())
}
