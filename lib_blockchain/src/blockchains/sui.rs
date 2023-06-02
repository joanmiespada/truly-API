use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use lib_config::{config::Config, environment::DEV_ENV, infra::restore_secret_key};
use log::{debug, error};
use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use url::Url;
use uuid::Uuid;

// use web3::{
//     contract::{tokens::Detokenize, Contract, Options},
//     transports::Http,
//     types::{Address, Block, BlockId, BlockNumber, H160, H256, U256},
//     Web3, //, signing::SecretKey,
// };

use crate::{
    errors::nft::HydrateMasterSecretKeyError,
    models::block_tx::BlockchainTx,
    repositories::{
        blockchain::BlockchainRepo, blockchain::BlockchainRepository, contract::ContractRepo,
        contract::ContractRepository,
    }, blockchains::sui_models::Response,
};
use crate::{errors::nft::NftUserAddressMalformedError, models::keypair::KeyPair};

const CONTRACT_METHOD_MINTING: &'static str = "add_hash";
const CONTRACT_METHOD_GET_CONTENT_BY_TOKEN: &'static str = "sui_getObject";
const BLOCKCHAIN_METHOD_EXEC: &'static str = "sui_executeTransactionBlock";
const BLOCKCHAIN_METHOD_ESTIMATE: &'static str = "sui_dryRunTransactionBlock";

use lib_licenses::errors::asset::AssetBlockachainError;

use super::chain::{ContentState, ContractContentInfo, NFTsRepository};

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[derive(Clone, Debug)]
pub struct SuiBlockChain {
    url: Url,
    contract_address: String,
    //contract_owner_address: String,
    //contract_owner_secret: String,
    contract_owner_cash: String,
    //kms_key_id: String,
    //aws: SdkConfig,
    config: Config,
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
            contract_address: contract.address().clone().unwrap().to_owned(),
            //contract_owner_address: contract.owner_address().clone().unwrap().to_owned(),
            //contract_owner_secret: contract.owner_secret().clone().unwrap().to_owned(),
            contract_owner_cash: contract.owner_cash().clone().unwrap().to_owned(),
            //kms_key_id: conf.env_vars().kms_key_id().to_owned(),
            //aws: conf.aws_config().to_owned(),
            config: conf.clone(),
            //blockhain_node_confirmations: blockchain.confirmations().to_owned(), //conf.env_vars().blockchain_confirmations().to_owned(),
            contract_id: aux.to_owned(), //contract.to_owned(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize,Clone )]
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
        use base64::{
            engine::{general_purpose},
            Engine as _,
        };
        use bcs::to_bytes;

        #[derive(Debug, Serialize, Deserialize)]
        struct Params {
            showInput: bool,
            showRawInput: bool,
            showEffects: bool,
            showEvents: bool,
            showObjectChanges: bool,
            showBalanceChanges: bool,
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct DataRequest {
            hash: String,
            hash_algorithm: String,
            asset_id: String,
        }

        let data = DataRequest {
            hash: hash_file.to_string(),
            hash_algorithm: hash_algorithm.to_string(),
            asset_id: asset_id.to_string(),
        };

        let tx_bytes = to_bytes(&&data)?;
        let tx_bytes_base64 = general_purpose::STANDARD.encode(tx_bytes);

        let aux = self.contract_owner_cash.clone();
        let tx_signatures = aux.as_bytes();
        let tx_signature_base64 =  general_purpose::STANDARD.encode( tx_signatures );

        let client = reqwest::Client::new();

        let params = Params {
            showInput: true,
            showRawInput: true,
            showEffects: true,
            showEvents: true,
            showObjectChanges: true,
            showBalanceChanges: true,
        };

        let payload = Payload {
            jsonrpc: "2.0".into(),
            id: 1,
            method:BLOCKCHAIN_METHOD_ESTIMATE.into(),
            params: vec![
                Value::String(tx_bytes_base64.into()),
                //Value::Array(vec![Value::String("AEZc4UMAoxzWtp+i1dvyOgmy+Eeb/5ZNwO5dpHBqX5Rt36+HhYnBby8asFU4b0i7TjQZGgLahT8w3NQUfk0NUQnqvbuA0Q1Bqu4RHV3JPpqmH+C527hWJGUBOZN1j9sg8w==".into())]),
                Value::Array(vec![Value::String( tx_signature_base64.into() )]),
                Value::Object(
                    serde_json::to_value(&params)
                        .unwrap()
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
                Value::String("WaitForLocalExecution".into()),
            ],
        };
        println!("{:#?}", payload);
        let res_op = client.post(self.url.clone()).json(&payload).send().await;
        if let Err(e)= res_op {
            error!("{}", e);
            return Err(e)?;
        }

        if let Err(e) = res_op.unwrap().json::<Response>().await {
            error!("{}", e);
            return Err(e)?;
        }


        let mut payload2 = payload.clone();
        payload2.method = BLOCKCHAIN_METHOD_EXEC .to_string();

        println!("{:#?}", payload2);
        let res_op = client.post(self.url.clone()).json(&payload2).send().await;
        if let Err(e)= res_op {
            error!("{}", e);
            return Err(e)?;
        }

        let tx = res_op.unwrap().json::<Response>().await?;




        //Estimate gas to consume
        // let estimate_call = contract.estimate_gas(
        //     CONTRACT_METHOD_MINTING,
        //     (to.clone(), token.clone(), hash_file.clone(), price.clone()),
        //     self.contract_owner_address.clone(), //self.contract_address.clone(), //account_owner,
        //     Options::default(),
        // );

        // let estimate_op = estimate_call.await;

        // let cost_gas: U256 = match estimate_op {
        //     Err(e) => {
        //         return Err(AssetBlockachainError(e.to_string()).into());
        //     }
        //     Ok(gas) => gas,
        // };
        // //request current gas price status
        // let gas_price_op = web3.eth().gas_price().await;
        // let gas_price: U256 = match gas_price_op {
        //     Err(e) => {
        //         return Err(AssetBlockachainError(e.to_string()).into());
        //     }
        //     Ok(gas) => gas,
        // };

        //let nonce = U256::from(1); //
        //let nonce = generate_nonce(&self.counter )?;
        // debug!("nonce calculated: {}", nonce);

        // let counter = web3
        //     .eth()
        //     .transaction_count(self.contract_address, None)
        //     .await
        //     .unwrap();
        // debug!("{}", counter);
        // //counter += U256::from_str_radix("1", 10).unwrap();
        //debug!("{}", counter);

        // let tx_options = Options {
        //     gas: Some(cost_gas), // Some(U256::from_str("400000").unwrap()), //250.000 weis  30.000.000 //with 400.000 gas units works!
        //     gas_price: Some(gas_price), // None,     // Some(U256::from_str("10000000").unwrap()), //10.000.000 weis
        //     value: None,
        //     condition: None,
        //     transaction_type: None,
        //     nonce: None, //Some(counter), // None
        //     access_list: None,
        //     max_fee_per_gas: None,
        //     max_priority_fee_per_gas: None,
        // };
        // debug!("calling from {}", self.contract_address.to_string());

        // let contract_owner_private_key;
        // //let contract_owner_private_key_op = self.decrypt_contract_owner_secret_key().await;
        // let contract_owner_private_key_op = SecretKey::from_str(
        //     restore_secret_key(
        //         self.contract_owner_secret.to_owned(),
        //         &self.kms_key_id,
        //         &self.config,
        //     )
        //     .await
        //     .unwrap()
        //     .as_str(),
        // );

        // match contract_owner_private_key_op {
        //     Err(_) => {
        //         return Err(HydrateMasterSecretKeyError {}.into());
        //     }
        //     Ok(value) => contract_owner_private_key = value,
        // }

        // let caller = contract.signed_call_with_confirmations(
        //     CONTRACT_METHOD_MINTING,
        //     (to.clone(), token.clone(), hash_file.clone(), price.clone()),
        //     tx_options,
        //     self.blockhain_node_confirmations.into(),
        //     &contract_owner_private_key,
        // );

        // let call_contract_op = caller.await;
        // let tx = match call_contract_op {
        //     Err(e) => {
        //         return Err(AssetBlockachainError(format!("{:?}", e)).into());
        //     }
        //     Ok(transact) => transact,
        // };
        // let tx_str = format!("blockchain: {} | tx: {:?} blockNum: {:?} gasUsed: {:?} w effectiveGasPrice: {:?} w  cost: {:?} gwei  from: {:?} to: {:?} ",
        //                                 self.url,
        //                                 Some(tx.transaction_hash),
        //                                 tx.block_number,
        //                                 tx.gas_used,
        //                                 tx.effective_gas_price,
        //                                 wei_to_gwei(tx.gas_used.unwrap() ) * wei_to_gwei( tx.effective_gas_price.unwrap()),
        //                                 Some(tx.from),
        //                                 tx.to );
        // // let block_num = match tx.block_number {
        //     None => None,
        //     Some(bn) => Some(bn.as_u64())
        // };
        let tx = tx.result;
        //tx.transaction.data.gasData.
        let tx_paylaod = BlockchainTx::new(
            asset_id.to_owned(),
            Utc::now(),
            Some(tx.digest),
            Some(0),              //tx.block_number,
            Some("".to_string()), //tx.gas_used,
            Some("".to_string()), // tx.effective_gas_price,
            Some(
                0.0, //wei_to_gwei(tx.gas_used.unwrap())
                    //    * wei_to_gwei(tx.effective_gas_price.unwrap_or_default()),
            ),
            Some("mist".to_string()),
            Some("".to_string()), //Some(tx.from),
            Some("".to_string()), //tx.to,
            self.contract_id,
            None,
        );
        Ok(tx_paylaod)
    }

    async fn get(&self, asset_id: &Uuid) -> ResultE<ContractContentInfo> {
        let token = asset_id.to_string();

        // let transport = web3::transports::Http::new(self.url.as_str()).unwrap();
        // let web3 = web3::Web3::new(transport);

        // //let contract_address = addr.clone();
        // let contract_op = Contract::from_json(
        //     web3.eth(),
        //     self.contract_address.clone(),
        //     include_bytes!("../../res/evm/LightNFT.abi"),
        // );
        // let contract = match contract_op {
        //     Err(e) => {
        //         return Err(AssetBlockachainError(e.to_string()).into());
        //     }
        //     Ok(cnt) => cnt,
        // };

        // let caller = contract.query(
        //     CONTRACT_METHOD_GET_CONTENT_BY_TOKEN,
        //     (token.clone(),),
        //     self.contract_address,
        //     Options::default(),
        //     None,
        // );
        // let call_contract_op: Result<ContractContentInfo, web3::contract::Error> = caller.await;

        #[derive(Debug, Serialize, Deserialize)]
        struct Params {
            showType: bool,
            showOwner: bool,
            showPreviousTransaction: bool,
            showDisplay: bool,
            showContent: bool,
            showBcs: bool,
            showStorageRebate: bool,
        }

        let params = Params {
            showType: true,
            showOwner: true,
            showPreviousTransaction: true,
            showDisplay: false,
            showContent: true,
            showBcs: false,
            showStorageRebate: true,
        };

        let payload = Payload {
            jsonrpc: "2.0".into(),
            id: 1,
            method: CONTRACT_METHOD_GET_CONTENT_BY_TOKEN.into(),
            params: vec![
                Value::String(token.into()),
                //Value::Array(vec![Value::String("AEZc4UMAoxzWtp+i1dvyOgmy+Eeb/5ZNwO5dpHBqX5Rt36+HhYnBby8asFU4b0i7TjQZGgLahT8w3NQUfk0NUQnqvbuA0Q1Bqu4RHV3JPpqmH+C527hWJGUBOZN1j9sg8w==".into())]),
                //Value::Array(vec![Value::String(self.contract_owner_secret.into())]),
                Value::Object(
                    serde_json::to_value(&params)
                        .unwrap()
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
                Value::String("WaitForLocalExecution".into()),
            ],
        };
        let client = reqwest::Client::new();
        let res = client.post(self.url.clone()).json(&payload).send().await?;

        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Debug)]
        pub struct RpcResponse {
            pub jsonrpc: String,
            pub result: ResultData,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct ResultData {
            pub data: ObjectData,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct ObjectData {
            pub objectId: String,
            pub version: String,
            pub digest: String,
            #[serde(rename = "type")]
            pub type_field: String,
            pub owner: Owner,
            pub previousTransaction: String,
            pub storageRebate: String,
            pub content: Content,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Owner {
            pub AddressOwner: String,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Content {
            pub dataType: String,
            #[serde(rename = "type")]
            pub type_field: String,
            pub hasPublicTransfer: bool,
            pub fields: Fields,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Fields {
            pub balance: String,
            pub id: Id,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Id {
            pub id: String,
        }

        let _tx = res.json::<RpcResponse>().await?;

        let res = ContractContentInfo {
            hashAlgo: "".to_string(),
            hashFile: "".to_string(),
            uri: "".to_string(),
            price: 0,
            state: ContentState::Active,
        };
        Ok(res)
    }
}

// fn _mist_to_sui(val: U256) -> f64 {
//     let res = val.as_u128() as f64;
//     res / 1_000_000_000_000_000_000.0
// }
// fn wei_to_gwei(wei_val: U256) -> f64 {
//     let res = wei_val.as_u128() as f64;
//     res / 1_000_000_000.0
//}

// pub async fn block_status(client: &Web3<Http>) -> Block<H256> {
//     let latest_block = client
//         .eth()
//         .block(BlockId::Number(BlockNumber::Latest))
//         .await
//         .unwrap()
//         .unwrap();

//     let timestamp = latest_block.timestamp.as_u64() as i64;
//     let naive = NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
//     let utc_dt: DateTime<Utc> = DateTime::from_utc(naive, Utc);

//     debug!(
//             "[{}] block num {}, parent {}, transactions: {}, gas used {}, gas limit {}, base fee {}, difficulty {}, total difficulty {}",
//             utc_dt.format("%Y-%m-%d %H:%M:%S"),
//             latest_block.number.unwrap(),
//             latest_block.parent_hash,
//             latest_block.transactions.len(),
//             latest_block.gas_used,
//             latest_block.gas_limit,
//             latest_block.base_fee_per_gas.unwrap_or_default(),
//             latest_block.difficulty,
//             latest_block.total_difficulty.unwrap()
//         );
//     return latest_block;
// }

// impl Detokenize for ContractContentInfo {
//     #[allow(non_snake_case)]
//     fn from_tokens(tokens: Vec<web3::ethabi::Token>) -> Result<Self, web3::contract::Error>
//     where
//         Self: Sized,
//     {
//         let mut hashFile = "".to_string();
//         let mut uri = "".to_string();
//         let mut state = ContentState::Active;
//         let mut price = 0;
//         let mut i = 0;
//         for token in tokens {
//             debug!("{:?}", token);
//             match i {
//                 0 => hashFile = token.into_string().unwrap(),
//                 1 => uri = token.into_string().unwrap(),
//                 2 => price = token.into_uint().unwrap().as_u64(),
//                 3 => state = ContentState::from_str(token.into_string().unwrap().as_str()).unwrap(),
//                 _ => {}
//             };
//             i += 1;
//         }
//         //let t = tokens[0].clone(); // .iter().next().unwrap().into_string().unwrap().clone();
//         Ok(Self {
//             hashFile,
//             uri,
//             price,
//             state,
//         })
//     }
// }
