pub mod notificate;

use aws_lambda_events::cloudwatch_events::CloudWatchEvent;
use lambda_runtime::LambdaEvent;
use lib_config::config::Config;
use lib_licenses::services::assets::{AssetService,AssetManipulation};
use lib_users::services::users::{UsersService, UserManipulation};
use serde_json::Value;
use std::{time::{Duration, SystemTime}, collections::HashMap};
use lib_engage::{
    repositories::{alert_similar::AlertSimilarRepo, subscription::SubscriptionRepo},
    services::{alert_similar::AlertSimilarService, subscription::SubscriptionService}
};

use crate::notificate::send_notifications;


#[derive(Debug)]
pub struct ApiLambdaError(pub String);

impl std::error::Error for ApiLambdaError {}

impl std::fmt::Display for ApiLambdaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "api lambda error: {}", self.0)
    }
}


//#[instrument]
pub async fn function_handler(
    _: LambdaEvent<CloudWatchEvent<Value>>,
    config: &Config,
    alert_service: &AlertSimilarService<AlertSimilarRepo>,
    subscription_service: &SubscriptionService<SubscriptionRepo>,
    user_service: &UsersService,
    asset_service: &AssetService,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    const ONE_HOUR_AND_HALF: Duration = Duration::from_secs(5400); //it must be little bit higher than the cronjob scheduled in terraform

    let mut buckets = HashMap::new();
    //let buckets: Vec<FoundSimilarContent> = Vec::new();

    let now = SystemTime::now();
    let (alerts, _token) = alert_service.get_latests_alerts(now, ONE_HOUR_AND_HALF ,None, None ).await?;
    
    for alert in &alerts{

        let asset_origen = asset_service.get_by_id( &alert.origin_asset_id().unwrap() ).await?;
        let asset_similar = asset_service.get_by_id( &alert.similar_asset_id().unwrap() ).await?;

        let subscriptions_origin = subscription_service.find_users_subscribed_to(asset_origen.id().to_owned()).await?;
        for subscription in subscriptions_origin{

            let user = user_service.get_by_id(&subscription).await?;
            //buckets.entry(user.email().unwrap()).or_insert(Vec::new()).push(asset_origen.url().unwrap());
            buckets.entry(user.email().clone() .unwrap() ).or_insert(HashMap::new()).entry(asset_origen.url().clone().unwrap()).or_insert(HashMap::new()).entry(asset_similar.url().clone().unwrap()).or_insert(alert.id().to_owned());
        }
        
        let subscriptions_similar = subscription_service.find_users_subscribed_to(asset_similar.id().to_owned()).await?;
        for subscription in subscriptions_similar{

            let user = user_service.get_by_id(&subscription).await?;
            //buckets.entry(user.email().unwrap()).or_insert(Vec::new()).push(asset_similar.url().unwrap());
             buckets.entry(user.email().clone().unwrap()).or_insert(HashMap::new()).entry(asset_similar.url().clone().unwrap()).or_insert(HashMap::new()).entry(asset_origen.url().clone().unwrap()).or_insert(alert.id().to_owned());

        }
        
    }

    let op = send_notifications(&config, buckets).await;
    if let Err(e) = op {
        log::error!("Could not send email: {e:?}") 
    }
    
    
    for alert in alerts{
        alert_service.delete(alert.id().to_owned()).await?;
    }

    
    Ok(())
    
}
