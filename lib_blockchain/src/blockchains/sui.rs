use async_trait::async_trait;
use base64::engine::general_purpose;
use base64::Engine;
use chrono::Utc;
use lib_config::infra::restore_secret_key;
use lib_config::{config::Config, environment::DEV_ENV};
use log::error;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use url::Url;
use uuid::Uuid;

use crate::models::keypair::KeyPair;
use crate::{
    errors::block_tx::BlockchainTxError,
    models::block_tx::BlockchainTx,
    repositories::{
        blockchain::BlockchainRepo, blockchain::BlockchainRepository, contract::ContractRepo,
        contract::ContractRepository,
    },
};

const CONTRACT_METHOD_MINTING: &'static str = "add_hash";

use super::chain::{ContractContentInfo, NFTsRepository};

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

use shared_crypto::intent::Intent;
use sui_json_rpc_types::{SuiObjectDataOptions, SuiParsedData, SuiTransactionBlockResponseOptions};
use sui_keys::keystore::{AccountKeystore, Keystore};
use sui_sdk::{
    json::SuiJsonValue,
    rpc_types::SuiTransactionBlockEffectsAPI,
    types::{
        base_types::{ObjectID, SuiAddress},
        transaction::Transaction,
    },
    SuiClientBuilder,
};
use sui_types::quorum_driver_types::ExecuteTransactionRequestType;

use zeroize::Zeroize;

#[derive(Clone, Debug)]
pub struct SuiBlockChain {
    url: Url,
    contract_address: String,
    contract_owner_address: String,
    contract_owner_secret: String,
    contract_owner_cash: String,
    config: Config,
    contract_id: u16,
}

impl SuiBlockChain {
    pub async fn new(
        conf: &Config,
        contracts_repo: &ContractRepo,
        blockchains_repo: &BlockchainRepo,
    ) -> ResultE<SuiBlockChain> {
        let aux = conf.env_vars().contract_id().unwrap();
        let contract = contracts_repo.get_by_id(&aux).await?;
        let blockchain = blockchains_repo.get_by_id(contract.blockchain()).await?;

        let blockchain_url;
        if conf.env_vars().environment().unwrap() == DEV_ENV {
            blockchain_url = blockchain.url().to_owned()
        } else {
            blockchain_url = Url::from_str(
                format!(
                    "{}/{}",
                    blockchain.url().to_owned(),
                    blockchain.api_key().to_owned()
                )
                .as_str(),
            )
            .unwrap();
        }

        Ok(SuiBlockChain {
            url: blockchain_url.to_owned(),
            contract_address: contract.address().clone().unwrap().to_owned(),
            contract_owner_address: contract.owner_address().clone().unwrap().to_owned(),
            contract_owner_secret: contract.owner_secret().clone().unwrap().to_owned(),
            contract_owner_cash: contract.owner_cash().clone().unwrap().to_owned(),
            config: conf.to_owned(),
            contract_id: aux.to_owned(),
        })
    }

    pub fn keystore_add_new_random_address(keystore: &mut Keystore) -> ResultE<String> {
        let (address, _phrase, _scheme) = keystore
            .generate_and_add_new_key(sui_types::crypto::SignatureScheme::ED25519, None, None)
            .unwrap();

        Ok(address.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Payload {
    jsonrpc: String,
    id: u32,
    method: String,
    params: Vec<serde_json::Value>,
}

#[async_trait]
impl NFTsRepository for SuiBlockChain {
    fn contract_id(&self) -> u16 {
        self.contract_id
    }
    async fn add(
        &self,
        asset_id: &Uuid,
        _: &KeyPair, //unused at SUI
        hash_file: &String,
        hash_algorithm: &String,
        _: &Option<u64>, //unused at SUI
        _: &u64,         //unused at SUI
    ) -> ResultE<BlockchainTx> {
        let sui = SuiClientBuilder::default()
            .build(self.url.as_str())
            .await
            .unwrap();

        let my_address = SuiAddress::from_str(&self.contract_owner_address.as_str())?;
        let gas_object_id = ObjectID::from_str(&self.contract_owner_cash.as_str())?;

        let package_object_id = ObjectID::from_str(self.contract_address.as_str())?;

        let module = "hasher";

        let function = CONTRACT_METHOD_MINTING;
        let gas_budget = 10000000;

        let transfer_tx_op = sui
            .transaction_builder()
            .move_call(
                my_address,
                package_object_id,
                module,
                function,
                vec![],
                vec![
                    SuiJsonValue::from_str(&hash_file.as_str())?,
                    SuiJsonValue::from_str(&hash_algorithm.as_str())?,
                    SuiJsonValue::from_str(&asset_id.to_string().as_str())?,
                ],
                Some(gas_object_id), //None,
                gas_budget,
            )
            .await;
        if let Err(err) = transfer_tx_op {
            error!("{}", err);
            return Err(BlockchainTxError { 0: err.to_string() }.into());
        }
        let transfer_tx = transfer_tx_op.ok().unwrap();

        // Sign transaction
        let kms_key_id = self.config.env_vars().kms_key_id().unwrap();
        let transaction_response_op;
        {
            let mut encoded_secret_cyphered = self.contract_owner_secret.clone();
            let mut encoded_secret_base64 =
                restore_secret_key(encoded_secret_cyphered.clone(), &kms_key_id, &self.config)
                    .await?;
            let mut contract_owner_secret =
                general_purpose::STANDARD_NO_PAD.decode(&encoded_secret_base64)?;

            let keystore: Keystore = bincode::deserialize(&contract_owner_secret[..]).unwrap();

            let signature =
                keystore.sign_secure(&my_address, &transfer_tx, Intent::sui_transaction())?;

            transaction_response_op = sui
                .quorum_driver_api()
                .execute_transaction_block(
                    Transaction::from_data(transfer_tx, Intent::sui_transaction(), vec![signature])
                        .verify()?,
                    SuiTransactionBlockResponseOptions::full_content(),
                    Some(ExecuteTransactionRequestType::WaitForLocalExecution),
                )
                .await;
            //clear memory with sensible data
            drop(keystore);
            contract_owner_secret.zeroize();
            encoded_secret_base64.zeroize();
            encoded_secret_cyphered.zeroize();
        }
        if let Err(err) = transaction_response_op {
            error!("{}", err);
            return Err(BlockchainTxError { 0: err.to_string() }.into());
        }
        let transaction_response = transaction_response_op.ok().unwrap();

        if let Some(confirmation) = transaction_response.confirmed_local_execution {
            if !confirmation {
                return Err(BlockchainTxError {
                    0: "failed transaction - confirmed local exec is false".to_string(),
                }
                .into());
            }
        }

        println!("{:#?}", transaction_response);

        let new_tx_address = transaction_response
            .clone()
            .effects
            .as_ref()
            .unwrap()
            .created()
            .first()
            .unwrap()
            .reference
            .object_id;

        let epoch = transaction_response
            .clone()
            .effects
            .as_ref()
            .unwrap()
            .executed_epoch();

        let gas_cost = transaction_response
            .clone()
            .balance_changes
            .unwrap()
            .first()
            .unwrap()
            .amount;

        let paid_from = transaction_response
            .clone()
            .object_changes
            .unwrap()
            .first()
            .unwrap()
            .object_id();

        let tx_paylaod = BlockchainTx::new(
            asset_id.to_owned(),
            Utc::now(),
            Some(new_tx_address.to_string()), //Some(tx.digest),
            Some(epoch),                      //tx.block_number,
            Some(gas_cost.to_string()),       //tx.gas_used,
            None,                             //Some("".to_string()), // tx.effective_gas_price,
            None,                             //Some(
            //gas_cost, //wei_to_gwei(tx.gas_used.unwrap())
            //    * wei_to_gwei(tx.effective_gas_price.unwrap_or_default()),
            //),
            Some("mist".to_string()),
            Some(paid_from.to_string()), //Some(tx.from),
            None,                        //Some("".to_string()), //tx.to,
            self.contract_id,
            None,
        );
        Ok(tx_paylaod)
    }
    //TODO
    async fn get(&self, token: &String) -> ResultE<ContractContentInfo> {
        let sui = SuiClientBuilder::default()
            .build(self.url.as_str())
            .await
            .unwrap();

        let token = SuiAddress::from_str(token.as_str())?;

        let transaction_response_op = sui
            .read_api()
            .get_object_with_options(
                ObjectID::from_address(token.into()),
                SuiObjectDataOptions::new().with_content(),
            )
            .await;
        if let Err(err) = transaction_response_op {
            error!("{}", err);
            return Err(BlockchainTxError { 0: err.to_string() }.into());
        }
        let objects = transaction_response_op.ok().unwrap();

        println!("{:?}", objects.data);
        #[derive(Deserialize, Debug)]
        struct Auxi {
            pub _hash: String,
            pub _algorithm: String,
            pub _truly_id: String,
        }
        impl<'a> From<&'a SuiParsedData> for Auxi {
            fn from(data: &'a SuiParsedData) -> Self {
                // implementation of the conversion logic goes here
                let _ppp = format!("{:?}", data);
                let _p: Auxi = serde_json::from_str(&_ppp).unwrap();

                Auxi {
                    _hash: "".to_string(),
                    _algorithm: "".to_string(),
                    _truly_id: "".to_string(),
                }
            }
        }

        // let _hash = objects.clone().data.unwrap().content.unwrap() ;// .first().content.unwrap().fields.unwrap();
        let _res: Auxi = objects
            .object()?
            .content
            .as_ref()
            .unwrap()
            .try_into()
            .unwrap();

        let res = ContractContentInfo {
            hashFile: "".to_string(),
            hashAlgo: "".to_string(),
            uri: Some("".to_string()),
            price: None,
            state: None,
            token: None,
        };

        Ok(res)
    }

    //we reuse the same keypair for all users and we don't want to store it (bool = false)
    async fn create_keypair(&self, _user_id: &String) -> ResultE<(KeyPair, bool)> {
        let user_key = KeyPair::new();

        Ok((user_key, false))
    }
}
