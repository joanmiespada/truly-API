use lib_async_ops::sns::{send as send_sns, SNSMessage};
use lib_config::config::Config;
use lib_licenses::services::nfts::{CreateNFTAsync, NFTsManipulation, NFTsService};
use tracing::{error, info, instrument};

const MAX_RETRIES: usize = 5;

#[instrument]
pub async fn async_minting(
    data: &mut CreateNFTAsync,
    config: &Config,
    blockchain_service: &NFTsService,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if data.get_tries() > MAX_RETRIES {
        //dicard minting
        info!("minting operation discarded! too many retries");
        info!("{}", data);
        let json_text = serde_json::to_string(&data)?;

        let message = SNSMessage {
            body: json_text.to_owned(),
        };
        let topic_arn = config.env_vars().topic_arn_mint_fails().to_owned();

        let op_sent = send_sns(config, &message, topic_arn.to_owned() ).await;
        match op_sent {
            Ok(_) => {
                info!("message notified at topic {}", topic_arn.to_owned());
            }
            Err(e) => {
                error!("message failed when posting in the topic {}", topic_arn.to_owned());
                error!("{}", data);
                error!("{}", e);
            }
        }
    } else {
        info!("miting...");
        info!("{}", data);
        let op_res = blockchain_service
            .try_mint(&data.asset_id, &data.user_id, &data.price)
            .await;

        match op_res {
            Err(e) => {
                error!("failed when minting, retring operation for later");
                error!("{}", e.to_string());
                //re-scheduele again
                data.increase_try();
                let json_text = serde_json::to_string(&data)?;

                let message = SNSMessage {
                    body: json_text.to_owned(),
                };
                let topic_arn = config.env_vars().topic_arn_mint_async().to_owned();

                let op_sent = send_sns(config, &message, topic_arn.to_owned()).await;
                match op_sent {
                    Ok(_) => {
                        error!("{}", data);
                    }
                    Err(e) => {
                        error!("message retry failed when posting in the topic {}", topic_arn.to_owned());
                        error!("{}", data);
                        error!("{}", e);
                    }
                }
            }
            Ok(tx) => {
                info!("minting successfully!");
                info!("{}",data);
                info!("{}", tx);
            }
        };
    }
    Ok(())
}
