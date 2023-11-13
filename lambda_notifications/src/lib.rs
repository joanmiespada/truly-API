pub mod notificate;

use aws_lambda_events::cloudwatch_events::CloudWatchEvent;
use lambda_runtime::LambdaEvent;
use lib_config::result::ResultE;
use lib_engage::{
    models::alert_similar::AlertSimilar,
    repositories::{
        alert_similar::AlertSimilarRepo, sender::SenderEmailsRepo, subscription::SubscriptionRepo,
    },
    services::{alert_similar::AlertSimilarService, subscription::SubscriptionService},
};
use lib_licenses::services::assets::{AssetManipulation, AssetService};
use lib_users::services::users::{UserManipulation, UsersService};
use serde_json::Value;
use std::collections::HashMap;
use url::Url;
use uuid::Uuid;

use crate::notificate::send_notifications;

pub type Similars = HashMap<Url, Uuid>;
pub type Subscription = HashMap<Url, Similars>;
pub type Notificator = HashMap<String, Subscription>;

pub async fn collect_alerts(
    alert_service: &AlertSimilarService<AlertSimilarRepo>,
    page_size: Option<u32>,
) -> ResultE<Vec<AlertSimilar>> {
    let mut all_alerts = Vec::new();
    let mut next_token: Option<String> = None;

    loop {
        let (alerts, token) = alert_service.get_all(next_token, page_size).await?;

        if alerts.is_empty() {
            break;
        }

        all_alerts.extend(alerts.into_iter());

        match token {
            Some(t) => next_token = Some(t),
            None => break,
        }
    }

    Ok(all_alerts)
}

pub async fn create_notifications(
    alerts: &Vec<AlertSimilar>,
    subscription_service: &SubscriptionService<SubscriptionRepo>,
    user_service: &UsersService,
    asset_service: &AssetService,
) -> ResultE<Notificator> {
    let mut buckets: Notificator = HashMap::new();

    for alert in alerts {
        let asset_origen_op = asset_service
            .get_by_id(&alert.origin_asset_id().unwrap())
            .await;

        let asset_origen = match asset_origen_op {
            Ok(a) => a,
            Err(e) => {
                log::error!("Error: Could not get asset: {e:?}");
                continue;
            }
        };
        let asset_similar_op = asset_service
            .get_by_id(&alert.similar_asset_id().unwrap())
            .await;

        let asset_similar = match asset_similar_op {
            Ok(a) => a,
            Err(e) => {
                log::error!("Error: Could not get asset: {e:?}");
                continue;
            }
        };

        let subscriptions_origin_op = subscription_service
            .find_users_subscribed_to(asset_origen.id().to_owned())
            .await;

        match subscriptions_origin_op {
            Ok(subscriptions_origin) => {
                for subscription in subscriptions_origin {
                    let user_op = user_service.get_by_id(&subscription.user_id).await;
                    if let Ok(user) = user_op {
                        //buckets.entry(user.email().unwrap()).or_insert(Vec::new()).push(asset_origen.url().unwrap());
                        buckets
                            .entry(user.email().clone().unwrap())
                            .or_insert(HashMap::new())
                            .entry(asset_origen.url().clone().unwrap())
                            .or_insert(HashMap::new())
                            .entry(asset_similar.url().clone().unwrap())
                            .or_insert(subscription.id.to_owned());
                    }
                }
            }
            Err(e) => {
                log::error!("Error: Could not get subscriptions for origin: {e:?}");
                //continue;
            }
        };

        let subscriptions_similar_op = subscription_service
            .find_users_subscribed_to(asset_similar.id().to_owned())
            .await;

        match subscriptions_similar_op {
            Ok(subscriptions_similar) => {
                for subscription in subscriptions_similar {
                    let user = user_service.get_by_id(&subscription.user_id).await?;
                    //buckets.entry(user.email().unwrap()).or_insert(Vec::new()).push(asset_similar.url().unwrap());
                    buckets
                        .entry(user.email().clone().unwrap())
                        .or_insert(HashMap::new())
                        .entry(asset_similar.url().clone().unwrap())
                        .or_insert(HashMap::new())
                        .entry(asset_origen.url().clone().unwrap())
                        .or_insert(subscription.id.to_owned());
                }
            }
            Err(e) => {
                log::error!("Error: Could not get subscriptions for similar: {e:?}");
                //continue;
            }
        }
    }
    Ok(buckets)
}

//#[instrument]
pub async fn function_handler(
    _: LambdaEvent<CloudWatchEvent<Value>>,
    alert_service: &AlertSimilarService<AlertSimilarRepo>,
    subscription_service: &SubscriptionService<SubscriptionRepo>,
    user_service: &UsersService,
    asset_service: &AssetService,
    sender_emails_repo: &SenderEmailsRepo,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let alerts = collect_alerts(alert_service, None).await?;

    let buckets =
        create_notifications(&alerts, subscription_service, user_service, asset_service).await?;

    let op = send_notifications(buckets, sender_emails_repo).await;
    if let Err(e) = op {
        log::error!("Could not send email: {e:?}")
    }

    for alert in alerts {
        alert_service.delete(alert.id().to_owned()).await?;
    }

    Ok(())
}
