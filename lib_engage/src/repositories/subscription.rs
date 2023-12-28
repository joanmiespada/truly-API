use crate::errors::subscription::SubscriptionError;
use crate::models::subscription::{ConfirmedStatus, Subscription};
use async_trait::async_trait;
use aws_sdk_dynamodb::types::{AttributeValue, Select};
use aws_sdk_dynamodb::Client;
use chrono::{
    prelude::{DateTime, Utc},
    Local,
};
use lib_config::config::Config;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

use super::schema_subscription::{
    ASSET_ID_FIELD, ASSET_USER_INDEX_ID, SUBSCRIPTION_ID_FIELD_PK, SUBSCRIPTION_TABLE_NAME,
    USER_ASSET_INDEX_ID, USER_ID_FIELD,
};
type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

pub const CREATION_TIME_FIELD_NAME: &str = "creationTime";
pub const LAST_UPDATE_TIME_FIELD_NAME: &str = "lastUpdateTime";
pub const CONFIRMED_FIELD_NAME: &str = "confirmationStatus";

#[async_trait]
pub trait SubscriptionRepository {
    async fn find_by_user(&self, user_id: String) -> ResultE<Vec<Subscription>>;
    async fn find_by_asset(&self, asset_id: Uuid) -> ResultE<Vec<Subscription>>;
    async fn add(&self, subscription: Subscription) -> ResultE<Uuid>;
    async fn get_by_id(&self, id: Uuid) -> ResultE<Option<Subscription>>;
    async fn get_by_user_asset_id(
        &self,
        asset_id: Uuid,
        user_id: String,
    ) -> ResultE<Option<Subscription>>;
    async fn delete(&self, id: Uuid) -> ResultE<()>;
    //async fn update_status(&self, id: Uuid, confirmed: ConfirmedStatus) -> ResultE<()>;
    async fn update(&self, id: Subscription) -> ResultE<()>;
    async fn check_exists(&self, user_id: String, asset_id: Uuid) -> ResultE<Option<uuid::Uuid>>;
}

#[derive(Clone, Debug)]
pub struct SubscriptionRepo {
    client: Client,
}

impl SubscriptionRepo {
    pub fn new(conf: &Config) -> SubscriptionRepo {
        SubscriptionRepo {
            client: Client::new(conf.aws_config()),
        }
    }

    fn map(data: HashMap<String, AttributeValue>) -> Subscription {
        let id = data.get(SUBSCRIPTION_ID_FIELD_PK).unwrap();
        let id1 = id.as_s().unwrap();
        let id2 = Uuid::from_str(id1).unwrap();

        let user_id = data.get(USER_ID_FIELD).unwrap();
        let user_id1 = user_id.as_s().unwrap();

        let asset_id = data.get(ASSET_ID_FIELD).unwrap();
        let asset_id1 = asset_id.as_s().unwrap();
        let asset_id2 = Uuid::from_str(asset_id1).unwrap();

        let creation_time = data.get(CREATION_TIME_FIELD_NAME).unwrap();
        let creation_time1 = creation_time.as_s().unwrap();
        let creation_time2 = DateTime::parse_from_rfc3339(creation_time1).unwrap();
        let creation_time3 = creation_time2.with_timezone(&Utc);

        let last_update_time = data.get(LAST_UPDATE_TIME_FIELD_NAME).unwrap();
        let last_update_time1 = last_update_time.as_s().unwrap();
        let last_update_time2 = DateTime::parse_from_rfc3339(last_update_time1).unwrap();
        let last_update_time3 = last_update_time2.with_timezone(&Utc);

        let status = data.get(CONFIRMED_FIELD_NAME).unwrap();
        let status1 = status.as_s().unwrap();
        let status2 = ConfirmedStatus::from_str(status1).unwrap();

        Subscription {
            id: id2,
            user_id: user_id1.to_string(),
            asset_id: asset_id2,
            confirmed: status2,
            creation_time: creation_time3,
            last_update_time: last_update_time3,
        }
    }
}

#[async_trait]
impl SubscriptionRepository for SubscriptionRepo {
    async fn find_by_user(&self, user_id: String) -> ResultE<Vec<Subscription>> {
        let user_id_av = AttributeValue::S(user_id.to_string());

        let confirmed_status_av = AttributeValue::S(ConfirmedStatus::Enabled.to_string());

        let request = self
            .client
            .query()
            .table_name(SUBSCRIPTION_TABLE_NAME.clone())
            .index_name(USER_ASSET_INDEX_ID)
            .key_condition_expression("#user_attr = :user_id")
            .filter_expression("#status_attr = :status_val")
            .expression_attribute_names("#user_attr".to_string(), USER_ID_FIELD.to_string())
            .expression_attribute_names(
                "#status_attr".to_string(),
                CONFIRMED_FIELD_NAME.to_string(),
            )
            .expression_attribute_values(":user_id", user_id_av)
            .expression_attribute_values(":status_val", confirmed_status_av)
            .select(Select::AllProjectedAttributes);

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                //tracing::error!(mssag);
                log::error!("{}", mssag);
                return Err(SubscriptionError::SubscriptionDynamoDBError(e.into()).into());
            }
            Ok(data) => {
                let aux = data.items();

                let mut subscriptions = Vec::new();
                for item in aux {
                    let subs = SubscriptionRepo::map(item.clone());

                    subscriptions.push(subs);
                }
                Ok(subscriptions)
            }
        }
    }

    async fn find_by_asset(&self, asset_id: Uuid) -> ResultE<Vec<Subscription>> {
        let asset_id_av = AttributeValue::S(asset_id.to_string());

        let confirmed_status_av = AttributeValue::S(ConfirmedStatus::Enabled.to_string());

        let request = self
            .client
            .query()
            .table_name(SUBSCRIPTION_TABLE_NAME.clone())
            .index_name(ASSET_USER_INDEX_ID)
            .key_condition_expression("#asset_attr = :asset_id")
            .filter_expression("#status_attr = :status_val")
            .expression_attribute_names("#asset_attr".to_string(), ASSET_ID_FIELD.to_string())
            .expression_attribute_names(
                "#status_attr".to_string(),
                CONFIRMED_FIELD_NAME.to_string(),
            )
            .expression_attribute_values(":asset_id", asset_id_av)
            .expression_attribute_values(":status_val", confirmed_status_av)
            .select(Select::AllProjectedAttributes);

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                //tracing::error!(mssag);
                log::error!("{}", mssag);
                return Err(SubscriptionError::SubscriptionDynamoDBError(e.into()).into());
            }
            Ok(data) => {
                let aux = data.items();

                let mut subscriptions = Vec::new();
                for item in aux {
                    let subs = SubscriptionRepo::map(item.clone());
                    //let doc = item.clone();
                    //let user_id = doc.get(USER_ID_FIELD).unwrap();
                    //let user_id1 = user_id.as_s().unwrap().clone();
                    subscriptions.push(subs);
                }
                Ok(subscriptions)
            }
        }
    }

    // async fn find_by_asset(&self, asset_id: Uuid) -> ResultE<Vec<String>> {
    //     let asset_id_prefix = format!("{}#", asset_id);
    //     let asset_id_prefix_av = AttributeValue::S(asset_id_prefix.to_string());

    //     let request = self
    //         .client
    //         .query()
    //         .table_name(SUBSCRIPTION_TABLE_NAME.clone())
    //         .index_name(SUBSCRIPTION_INDEX_ID.clone())
    //         .key_condition_expression("begins_with(#asset_attr, :asset_id_prefix)")
    //         .expression_attribute_names("#asset_attr".to_string(), ASSET_ID_FIELD.to_string())
    //         .expression_attribute_values(":asset_id_prefix", asset_id_prefix_av)
    //         .select(Select::AllProjectedAttributes);

    //     let results = request.send().await;
    //     match results {
    //         Err(e) => {
    //             let mssag = format!(
    //                 "Error at [{}] - {} ",
    //                 Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
    //                 e
    //             );
    //             //tracing::error!(mssag);
    //             log::error!("{}", mssag);
    //             return Err(SubscriptionError::SubscriptionDynamoDBError(e.into()).into());
    //         }
    //         Ok(data) => {
    //             let op_items = data.items();
    //             match op_items {
    //                 None => {
    //                     return Err(SubscriptionError::AssetNotFound(asset_id).into());
    //                 }
    //                 Some(aux) => {
    //                     let mut subscriptions = Vec::new();
    //                     for item in aux {
    //                         let doc = item.clone();
    //                         let ass1_id = doc.get(USER_ID_FIELD).unwrap();
    //                         let ass1_id1 = ass1_id.as_s().unwrap();
    //                         subscriptions.push(ass1_id1.clone());
    //                     }
    //                     Ok(subscriptions)
    //                 }
    //             }
    //         }
    //     }
    // }

    async fn add(&self, subs: Subscription) -> ResultE<Uuid> {
        let subs_id_av = AttributeValue::S(subs.id.to_string());
        // let pk_av = AttributeValue::S(format!(
        //     "{}#{}",
        //     subs.user_id.to_string(),
        //     subs.asset_id.to_string()
        // ));
        let creation_time_av = AttributeValue::S(subs.creation_time.to_rfc3339());
        let last_update_time_av = AttributeValue::S(subs.last_update_time.to_rfc3339());
        let asset_id_av = AttributeValue::S(subs.asset_id.to_string());
        let user_id_av = AttributeValue::S(subs.user_id.to_string());
        let status_av = AttributeValue::S(subs.confirmed.to_string());

        let request = self
            .client
            .put_item()
            .table_name(SUBSCRIPTION_TABLE_NAME.clone())
            .item(SUBSCRIPTION_ID_FIELD_PK, subs_id_av)
            .item(CREATION_TIME_FIELD_NAME, creation_time_av)
            .item(LAST_UPDATE_TIME_FIELD_NAME, last_update_time_av)
            .item(ASSET_ID_FIELD, asset_id_av)
            .item(USER_ID_FIELD, user_id_av)
            .item(CONFIRMED_FIELD_NAME, status_av);

        match request.send().await {
            Ok(_) => Ok(subs.id),
            Err(e) => {
                let message = format!("Error creating license: {}", e);
                //tracing::error!("{}", message);
                log::error!("{}", message);
                Err(SubscriptionError::SubscriptionDynamoDBError(e.into()).into())
            }
        }
    }

    async fn get_by_id(&self, id: Uuid) -> ResultE<Option<Subscription>> {
        let subs_id_av = AttributeValue::S(id.to_string());

        let request = self
            .client
            //.query()
            .get_item()
            .table_name(SUBSCRIPTION_TABLE_NAME.clone())
            //.index_name(SUBSCRIPTION_INDEX_ID.clone())
            .key(SUBSCRIPTION_ID_FIELD_PK, subs_id_av);
        //.key_condition_expression("#asset_attr = :asset_id")
        //.expression_attribute_names(
        //    "#asset_attr".to_string(),
        //    SUBSCRIPTION_ID_FIELD.to_string(),
        // )
        // .expression_attribute_values(":asset_id", subs_id_av)
        // .select(Select::AllProjectedAttributes);

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                //tracing::error!(mssag);
                log::error!("{}", mssag);
                return Err(SubscriptionError::SubscriptionDynamoDBError(e.into()).into());
            }
            Ok(data) => {
                if let Some(item) = data.item() {
                    if !item.is_empty() {
                        //let aux = &items[0];
                        let res = SubscriptionRepo::map(item.clone());
                        Ok(Some(res))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
        }
    }

    async fn get_by_user_asset_id(
        &self,
        asset_id: Uuid,
        user_id: String,
    ) -> ResultE<Option<Subscription>> {
        let user_id_av = AttributeValue::S(user_id.to_string());
        let asset_id_av = AttributeValue::S(asset_id.to_string());

        let request = self
            .client
            //.get_item()
            .query()
            .table_name(SUBSCRIPTION_TABLE_NAME.clone())
            .index_name(USER_ASSET_INDEX_ID)
            //.key(SUBSCRIPTION_FIELD_PK, subs_id_av);
            .key_condition_expression("#user_attr = :user_id AND #asset_attr = :asset_id")
            .expression_attribute_names("#asset_attr".to_string(), ASSET_ID_FIELD.to_string())
            .expression_attribute_names("#user_attr".to_string(), USER_ID_FIELD.to_string())
            .expression_attribute_values(":asset_id", asset_id_av)
            .expression_attribute_values(":user_id", user_id_av)
            .select(Select::AllProjectedAttributes);

        let results = request.send().await;
        if let Err(e) = results {
            let mssag = format!(
                "Error at [{}] - {} ",
                Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                e
            );
            //tracing::error!(mssag);
            log::error!("{}", mssag);
            return Err(SubscriptionError::SubscriptionDynamoDBError(e.into()).into());
        }

        let aux = results.unwrap();
        let aux2 = aux.items();
        if aux2.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(SubscriptionRepo::map(aux2[0].clone())))
        }
    }

    async fn delete(&self, id: Uuid) -> ResultE<()> {
        //let aux = self.get_by_id(id).await?;

        //let key = format!("{}#{}", aux.user_id.to_string(), aux.asset_id.to_string());

        let key_av = AttributeValue::S(id.to_string());

        let request = self
            .client
            .delete_item()
            .table_name(SUBSCRIPTION_TABLE_NAME.clone())
            .key(SUBSCRIPTION_ID_FIELD_PK, key_av);

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                //tracing::error!(mssag);
                log::error!("{}", mssag);
                return Err(SubscriptionError::SubscriptionDynamoDBError(e.into()).into());
            }
            Ok(_) => Ok(()),
        }
    }

    // async fn update_status(&self, id: Uuid, confirmed: ConfirmedStatus) -> ResultE<()> {
    //     let mut aux = self.get_by_id(id).await?;
    //     aux.confirmed = confirmed;
    //     self.update(aux).await
    // }

    async fn update(&self, subs: Subscription) -> ResultE<()> {
        //let aux = self.get_by_id(subs.id).await?;

        //let key = format!("{}#{}", aux.user_id.to_string(), aux.asset_id.to_string());

        let key_av = AttributeValue::S(subs.id.to_string());

        let confirmed_av = AttributeValue::S(subs.confirmed.to_string());

        let request = self
            .client
            .update_item()
            .table_name(SUBSCRIPTION_TABLE_NAME.clone())
            .key(SUBSCRIPTION_ID_FIELD_PK, key_av)
            .update_expression(format!("SET {} = :value", CONFIRMED_FIELD_NAME))
            .expression_attribute_values(":value", confirmed_av);

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                //tracing::error!(mssag);
                log::error!("{}", mssag);
                return Err(SubscriptionError::SubscriptionDynamoDBError(e.into()).into());
            }
            Ok(_) => Ok(()),
        }
    }

    async fn check_exists(&self, user_id: String, asset_id: Uuid) -> ResultE<Option<uuid::Uuid>> {
        // Implement the logic to check if a certain subscription exists based on user ID and asset ID.

        let aux = self.get_by_user_asset_id(asset_id, user_id).await;
        match aux {
            Ok(aux) => match aux {
                None => Ok(None),
                Some(aux) => Ok(Some(aux.id)),
            },
            Err(_) => Ok(None),
        }
    }
}
