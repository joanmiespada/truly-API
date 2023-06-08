use async_trait::async_trait;
use chrono::Utc;
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
use sui_json_rpc_types::{
    SuiObjectDataOptions, SuiParsedData,
    SuiTransactionBlockResponseOptions,
};
use sui_keys::keystore::{AccountKeystore, Keystore};
use sui_sdk::{
    json::SuiJsonValue,
    rpc_types::SuiTransactionBlockEffectsAPI,
    types::{
        base_types::{ObjectID, SuiAddress},
        //id::UID,
        transaction::Transaction,
    },
    //SuiClient,
    SuiClientBuilder,
};
use sui_types::quorum_driver_types::ExecuteTransactionRequestType;

use sui_types::base_types::{SUI_ADDRESS_LENGTH};

use fastcrypto::hash::{ HashFunction, Blake2b256 };



#[derive(Clone, Debug)]
pub struct SuiBlockChain {
    url: Url,
    contract_address: String,
    contract_owner_address: String,
    contract_owner_secret: String,
    contract_owner_cash: String,
    //kms_key_id: String,
    //aws: SdkConfig,
    //blockhain_node_confirmations: u16,
    contract_id: u16,
}

impl SuiBlockChain {
    pub async fn new(
        conf: &Config,
        contracts_repo: &ContractRepo,
        blockchains_repo: &BlockchainRepo,
    ) -> ResultE<SuiBlockChain> {
        let aux = conf.env_vars().contract_id();
        let contract = contracts_repo.get_by_id(&aux).await?;
        let blockchain = blockchains_repo.get_by_id(contract.blockchain()).await?;

        let blockchain_url;
        if conf.env_vars().environment() == DEV_ENV {
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
            //kms_key_id: conf.env_vars().kms_key_id().to_owned(),
            contract_address: contract.address().clone().unwrap().to_owned(),
            contract_owner_address: contract.owner_address().clone().unwrap().to_owned(),
            contract_owner_secret: contract.owner_secret().clone().unwrap().to_owned(),
            contract_owner_cash: contract.owner_cash().clone().unwrap().to_owned(),
            //kms_key_id: conf.env_vars().kms_key_id().to_owned(),
            //aws: conf.aws_config().to_owned(),
            //config: conf.clone(),
            //blockhain_node_confirmations: blockchain.confirmations().to_owned(), //conf.env_vars().blockchain_confirmations().to_owned(),
            contract_id: aux.to_owned(), //contract.to_owned(),
        })
    }

    pub fn keystore_to_address(keystore: &Keystore) -> ResultE<String> {
        let pubkey = keystore.keys()[0].clone();

        let mut hasher = Blake2b256::new(); //  DefaultHash::default();
        hasher.update([pubkey.flag()]);
        hasher.update(pubkey);
        let g_arr = hasher.finalize();
        let mut res = [0u8; SUI_ADDRESS_LENGTH];
        res.copy_from_slice(&AsRef::<[u8]>::as_ref(&g_arr)[..SUI_ADDRESS_LENGTH]);
        let addr = SuiAddress::try_from(res.as_slice())?;
        let addr = addr.to_string();
        Ok(addr)
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
        _user_key: &KeyPair,
        hash_file: &String,
        hash_algorithm: &String,
        _prc: &Option<u64>,
        _cntr: &u64,
    ) -> ResultE<BlockchainTx> {
        let sui = SuiClientBuilder::default()
            .build(self.url.as_str())
            .await
            .unwrap();

        // let keystore_path = match dirs::home_dir() {
        //     Some(v) => v.join(".sui").join("sui_config").join("sui.keystore"),
        //     None => panic!("Cannot obtain home directory path"),
        // };

        //let keystore = Keystore::File(FileBasedKeystore::new(&keystore_path)?);

        let my_address = SuiAddress::from_str(
            //"0x042d9857b31cdec48b00332fec4a7adf8bf8e2a5a1561ef7778ce1abf7b91f30",
            &self.contract_owner_address.as_str(),
        )?;
        let gas_object_id = ObjectID::from_str(
            //"0x1b06b6b809dffa47c03bcef9d4375ca835c2b2053618150b6f54789e4e2c0163",
            &self.contract_owner_cash.as_str(),
        )?;

        let package_object_id = ObjectID::from_str(
            //"0xdd99302b8971ca07d516bea8b09e560ec1b72cc6c722f5f7b5a9f6c6fb1cff29",
            self.contract_address.as_str(),
        )?;

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
            //return Err(err)?;
            return Err(BlockchainTxError { 0: err.to_string() }.into());
        }
        let transfer_tx = transfer_tx_op.ok().unwrap();

        // Sign transaction
        //let keystore = Keystore::from(FileBasedKeystore::new(&keystore_path)?);

       let encoded_secret= self.contract_owner_secret.clone().into_bytes();

        let keystore: Keystore = bincode::deserialize(&encoded_secret[..]).unwrap();

        let signature =
            keystore.sign_secure(&my_address, &transfer_tx, Intent::sui_transaction())?;

        let transaction_response_op = sui
            .quorum_driver_api()
            .execute_transaction_block(
                Transaction::from_data(transfer_tx, Intent::sui_transaction(), vec![signature])
                    .verify()?,
                SuiTransactionBlockResponseOptions::full_content(),
                Some(ExecuteTransactionRequestType::WaitForLocalExecution),
            )
            .await;
        if let Err(err) = transaction_response_op {
            error!("{}", err);
            //return Err(err)?;
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

        //tx.transaction.data.gasData.
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
            //return Err(err)?;
            return Err(BlockchainTxError { 0: err.to_string() }.into());
        }
        let objects = transaction_response_op.ok().unwrap();

        println!("{:?}", objects.data);
        #[derive(Deserialize, Debug)]
        struct Auxi {
            pub hash: String,
            pub algorithm: String,
            pub truly_id: String,
        }
        impl<'a> From<&'a SuiParsedData> for Auxi {
            fn from(data: &'a SuiParsedData) -> Self {
                // implementation of the conversion logic goes here
                let _ppp = format!("{:?}", data);
                let _p: Auxi = serde_json::from_str(&_ppp).unwrap();

                Auxi {
                    hash: "".to_string(),
                    algorithm: "".to_string(),
                    truly_id: "".to_string(),
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
    async fn create_keypair(&self, user_id: &String) -> ResultE<(KeyPair, bool)> {
        /*
                use secp256k1::rand::{rngs, SeedableRng};
                use web3::signing::keccak256;

                let secp = secp256k1::Secp256k1::new();

                //let mut rng = rand_hc::Hc128Rng::from_entropy();
                let mut rng = rngs::StdRng::seed_from_u64(rand::random::<u64>());

                let contract_owner_key_pair = secp.generate_keypair(&mut rng);
                let contract_owner_public = contract_owner_key_pair.1.serialize();
                let hash = keccak256(&contract_owner_public[1..32]);
                let user_address = format!("0x{}", hex::encode(&hash[12..32]));
                //let user_private = contract_owner_key_pair.0;
                let user_private_key = format!("{}", contract_owner_key_pair.0.display_secret());
                let user_public_key = format!("{}", contract_owner_key_pair.1);

                let user_private_key_cyphered = store_secret_key(&user_private_key, &self.kms_key_id, &self.config).await?;
                let user_public_key_cyphered = store_secret_key(&user_public_key, &self.kms_key_id, &self.config).await?;
        */
        let mut user_key = KeyPair::new();
        user_key.set_user_id(user_id);
        //user_key.set_address(&user_address);
        //user_key.set_private_key(&user_private_key_cyphered);
        //user_key.set_public_key(&user_public_key_cyphered);

        Ok((user_key, false))
    }
}
