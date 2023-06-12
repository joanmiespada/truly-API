use std::{collections::HashMap, str::FromStr};

use async_trait::async_trait;
use aws_sdk_dynamodb::types::builders::PutBuilder;
use aws_sdk_dynamodb::types::{AttributeValue, Put, Select, TransactWriteItem};
use chrono::{
    prelude::{DateTime, Utc},
    Local,
};
use lib_config::config::Config;
//use web3::types::H160;

use crate::{
    errors::contract::{ContractDynamoDBError, ContractNoExistsError},
    models::contract::{Contract, ContractStatus},
};

use super::schema_contract::{
    CONTRACT_BLOCKCHAIN_FIELD, CONTRACT_BLOCKCHAIN_INDEX, CONTRACT_ID_FIELD_PK,
    CONTRACT_STATUS_FIELD_NAME, CONTRACT_TABLE_NAME,
};

pub const CREATIONTIME_FIELD_NAME: &str = "creationTime";
pub const CONTRACT_ADDRESS_FIELD_NAME: &str = "address";
pub const CONTRACT_OWNER_ADDRESS_FIELD_NAME: &str = "owner_address";
pub const CONTRACT_OWNER_SECRET_FIELD_NAME: &str = "owner_secret";
pub const CONTRACT_OWNER_CASH_FIELD_NAME: &str = "owner_cash";
pub const CONTRACT_DETAILS_FIELD_NAME: &str = "details";

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[async_trait]
pub trait ContractRepository {
    async fn add(&self, cont: &Contract) -> ResultE<()>;
    async fn update(&self, cont: &Contract) -> ResultE<()>;
    async fn get_by_id(&self, id: &u16) -> ResultE<Contract>;
    async fn get_by_blockchain(&self, blockchain: &String) -> ResultE<Contract>;
}

#[derive(Clone, Debug)]
pub struct ContractRepo {
    client: aws_sdk_dynamodb::Client,
}

impl ContractRepo {
    pub fn new(conf: &Config) -> ContractRepo {
        ContractRepo {
            client: aws_sdk_dynamodb::Client::new(conf.aws_config()),
        }
    }
    fn new_or_update(&self, contract: &Contract) -> ResultE<PutBuilder> {
        let id_av = AttributeValue::N(contract.id().to_string());
        let blockchain_av = AttributeValue::S(contract.blockchain().to_string());
        let status_av = AttributeValue::S(contract.status().to_string());
        let creation_time_av = AttributeValue::S(iso8601(contract.creation_time()));

        let mut items = Put::builder();

        items = items
            .item(CONTRACT_ID_FIELD_PK, id_av)
            .item(CONTRACT_BLOCKCHAIN_FIELD, blockchain_av)
            .item(CONTRACT_STATUS_FIELD_NAME, status_av)
            .item(CREATIONTIME_FIELD_NAME, creation_time_av);

        if let Some(val) = contract.address() {
            let av = AttributeValue::S(val.clone());
            items = items.item(CONTRACT_ADDRESS_FIELD_NAME, av)
        }
        if let Some(val) = contract.owner_address() {
            let av = AttributeValue::S(val.clone());
            items = items.item(CONTRACT_OWNER_ADDRESS_FIELD_NAME, av)
        }
        if let Some(val) = contract.owner_secret() {
            let av = AttributeValue::S(val.clone());
            items = items.item(CONTRACT_OWNER_SECRET_FIELD_NAME, av)
        }
        if let Some(val) = contract.owner_cash() {
            let av = AttributeValue::S(val.clone());
            items = items.item(CONTRACT_OWNER_CASH_FIELD_NAME, av)
        }
        if let Some(val) = contract.details() {
            let av = AttributeValue::S(val.clone());
            items = items.item(CONTRACT_DETAILS_FIELD_NAME, av)
        }

        Ok(items)
    }
}

#[async_trait]
impl ContractRepository for ContractRepo {
    async fn add(&self, contract: &Contract) -> ResultE<()> {
        let items = self.new_or_update(contract).unwrap();

        let request = self.client.transact_write_items().transact_items(
            TransactWriteItem::builder()
                .put(items.table_name(CONTRACT_TABLE_NAME).build())
                .build(),
        );

        match request.send().await {
            Ok(_) => Ok(()),
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(ContractDynamoDBError(e.to_string()).into());
            }
        }
    }
    async fn update(&self, contract: &Contract) -> ResultE<()> {
        let items = self.new_or_update(contract).unwrap();

        let request = self.client.transact_write_items().transact_items(
            TransactWriteItem::builder()
                .put(items.table_name(CONTRACT_TABLE_NAME).build())
                .build(),
        );

        match request.send().await {
            Ok(_) => Ok(()),
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(ContractDynamoDBError(e.to_string()).into());
            }
        }
    }
    async fn get_by_id(&self, id: &u16) -> ResultE<Contract> {
        let id_av = AttributeValue::N(id.to_string());
        let request = self
            .client
            .get_item()
            .table_name(CONTRACT_TABLE_NAME)
            .key(CONTRACT_ID_FIELD_PK, id_av.clone());

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(ContractDynamoDBError(e.to_string()).into());
            }
            Ok(_) => {}
        }
        match results.unwrap().item {
            None => Err(ContractNoExistsError("id doesn't exist".to_string()).into()),
            Some(aux) => {
                let mut cont = Contract::new();

                mapping_from_doc_to_contract(&aux, &mut cont);

                Ok(cont)
            }
        }
    }

    async fn get_by_blockchain(&self, blockchain: &String) -> ResultE<Contract> {
        let block_ch_av = AttributeValue::S(blockchain.to_owned());
        let state_ch_av = AttributeValue::S(ContractStatus::Enabled.to_string());
        let filter = format!(
            "{} = :value, {} = :state",
            CONTRACT_BLOCKCHAIN_FIELD, CONTRACT_STATUS_FIELD_NAME
        );

        let request = self
            .client
            .query()
            .table_name(CONTRACT_TABLE_NAME)
            .index_name(CONTRACT_BLOCKCHAIN_INDEX)
            .key_condition_expression(filter)
            .expression_attribute_values(":value".to_string(), block_ch_av)
            .expression_attribute_values(":state".to_string(), state_ch_av)
            .select(Select::AllAttributes);

        let results = request.send().await;
        match results {
            Err(e) => {
                let mssag = format!(
                    "Error at [{}] - {} ",
                    Local::now().format("%m-%d-%Y %H:%M:%S").to_string(),
                    e
                );
                tracing::error!(mssag);
                return Err(ContractDynamoDBError(e.to_string()).into());
            }
            Ok(items) => {
                let docus = items.items().unwrap();

                if docus.len() != 1 {
                    return Err(ContractDynamoDBError(format!(
                        "too many contracts enabled, don't which contract use. blockchain: {}",
                        blockchain
                    ))
                    .into());
                }
                let doc = docus.first().unwrap();
                let mut contract_f = Contract::new();
                mapping_from_doc_to_contract(doc, &mut contract_f);

                Ok(contract_f)
            }
        }
    }
}

fn iso8601(st: &DateTime<Utc>) -> String {
    let dt: DateTime<Utc> = st.clone().into();
    format!("{}", dt.format("%+"))
}

fn from_iso8601(st: &String) -> DateTime<Utc> {
    let aux = st.parse::<DateTime<Utc>>().unwrap();
    aux
}

pub fn mapping_from_doc_to_contract(
    doc: &HashMap<String, AttributeValue>,
    contract: &mut Contract,
) {
    let id = doc.get(CONTRACT_ID_FIELD_PK).unwrap();
    let contract_id = id.as_n().unwrap();
    //let uuid = Uuid::from_str(contract_id).unwrap();
    let iid = u16::from_str(&contract_id).unwrap();
    contract.set_id(&iid);

    let blockchain = doc.get(CONTRACT_BLOCKCHAIN_FIELD).unwrap();
    let blockchain1 = blockchain.as_s().unwrap();
    contract.set_blockchain(blockchain1);

    if let Some(value) = doc.get(CONTRACT_ADDRESS_FIELD_NAME) {
        let value1 = value.as_s().unwrap();
        contract.set_address(&value1);
    }

    if let Some(value) = doc.get(CONTRACT_OWNER_ADDRESS_FIELD_NAME) {
        let value1 = value.as_s().unwrap();
        contract.set_owner_address(&value1);
    }

    if let Some(value) = doc.get(CONTRACT_OWNER_SECRET_FIELD_NAME) {
        let value1 = value.as_s().unwrap();
        contract.set_owner_secret(&value1);
    }

    if let Some(value) = doc.get(CONTRACT_OWNER_CASH_FIELD_NAME) {
        let value1 = value.as_s().unwrap();
        contract.set_owner_cash(&value1);
    }

    if let Some(value) = doc.get(CONTRACT_DETAILS_FIELD_NAME) {
        let value1 = value.as_s().unwrap();
        contract.set_details(&value1);
    }

    let status = doc.get(CONTRACT_STATUS_FIELD_NAME).unwrap();
    let status1 = status.as_s().unwrap();
    let sts = ContractStatus::from_str(status1).unwrap();
    contract.set_status(&sts);

    if let Some(creation_time) = doc.get(CREATIONTIME_FIELD_NAME) {
        contract.set_creation_time(&from_iso8601(creation_time.as_s().unwrap()));
    }
}
