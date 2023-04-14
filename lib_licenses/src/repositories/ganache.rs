use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use lib_config::{
    config::Config, environment::DEV_ENV, infra::restore_secret_key
};
use log::debug;
use mac_address::get_mac_address;
use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};
use snowflake::SnowflakeIdGenerator;
use std::{
    fmt,
    str::FromStr,
    sync::{Arc, Mutex},
};
use url::Url;
use uuid::Uuid;

use web3::{
    contract::{tokens::Detokenize, Contract, Options},
    transports::Http,
    types::{Address, Block, BlockId, BlockNumber, H160, H256, U256},
    Web3,
};

use crate::{errors::nft::HydrateMasterSecretKeyError, models::block_tx::BlockchainTx};
use crate::{
    errors::nft::{NftBlockChainNonceMalformedError, NftUserAddressMalformedError},
    models::keypair::KeyPair,
};

const CONTRACT_METHOD_MINTING: &'static str = "mint";
const CONTRACT_METHOD_GET_CONTENT_BY_TOKEN: &'static str = "getContentByToken";

use crate::errors::asset::AssetBlockachainError;

use super::{
    blockchain::{BlockchainRepo, BlockchainRepository},
    contract::{ContractRepo, ContractRepository},
};

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;
#[async_trait]
pub trait NFTsRepository {
    async fn add(
        &self,
        asset_id: &Uuid,
        user_key: &KeyPair,
        hash_file: &String,
        price: &u64,
        counter: &u64,
    ) -> ResultE<BlockchainTx>;

    async fn get(&self, asset_id: &Uuid) -> ResultE<GanacheContentInfo>;
    fn contract_id(&self) -> u16;
}

#[derive(Clone, Debug)]
pub struct GanacheRepo {
    url: Url,
    contract_address: Address,
    contract_owner_address: Address,
    contract_owner_secret: String,
    kms_key_id: String,
    //aws: SdkConfig,
    config: Config,
    blockhain_node_confirmations: u16,
    contract_id: u16,
}

impl GanacheRepo {
    pub async fn new(
        conf: &Config,
        contracts_repo: &ContractRepo,
        blockchains_repo: &BlockchainRepo,
    ) -> ResultE<GanacheRepo> {
        //TODO: read from contracts table!
        /*
                let contract_address_position;
                let aux = conf.env_vars().contract_address();
                let contract_address_position_op = Address::from_str(aux.as_str()); //.unwrap();
                match contract_address_position_op {
                    Err(e) => {
                        return Err(e.to_string());
                    }
                    Ok(val) => contract_address_position = val,
                }
                let contract_owner_position;
                let aux2 = conf.env_vars().contract_owner_address();
                let contract_owner_position_op = Address::from_str(aux2.as_str()); //.unwrap();
                match contract_owner_position_op {
                    Err(e) => return Err(e.to_string()),
                    Ok(val) => contract_owner_position = val,
                }



        */
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

        Ok(GanacheRepo {
            url: blockchain_url.to_owned(),
            contract_address: contract.address().unwrap().to_owned(), //contract_address_position,
            contract_owner_address: contract.owner_address().unwrap().to_owned(), //contract_owner_position,
            contract_owner_secret: contract.owner_secret().clone().unwrap().to_owned(), //contract_owner_position,
            kms_key_id: conf.env_vars().kms_key_id().to_owned(),
            //aws: conf.aws_config().to_owned(),
            config: conf.clone(),
            blockhain_node_confirmations: blockchain.confirmations().to_owned(), //conf.env_vars().blockchain_confirmations().to_owned(),
            contract_id: aux.to_owned(), //contract.to_owned(),
        })
    }

    /*
    async fn decrypt_contract_owner_secret_key(&self) -> ResultE<SecretKey> {
        use base64::{engine::general_purpose, Engine as _};

        // let client = aws_sdk_secretsmanager::Client::new(&self.aws);
        // let scr = client
        //     .get_secret_value()
        //     .secret_id(SECRETS_MANAGER_SECRET_KEY)
        //     .send()
        //     .await?;

        // let secret_key_cyphered_b64 = scr.secret_string().unwrap();
        let secret_key_cyphered_b64 = self.contract_owner_secret.to_owned();


        let secret_key_cyphered = general_purpose::STANDARD
            .decode(secret_key_cyphered_b64)
            .unwrap();

        let data = aws_sdk_kms::types::Blob::new(secret_key_cyphered);

        let client = aws_sdk_kms::Client::new(&self.aws);

        let resp;
        let resp_op = client
            .decrypt()
            .key_id(self.kms_key_id.to_owned())
            .ciphertext_blob(data)
            .send()
            .await;
        match resp_op {
            Err(e) => {
                return Err(e.into());
            }
            Ok(val) => resp = val,
        }

        let inner = resp.plaintext.unwrap();
        let bytes = inner.as_ref();

        let secret_key_raw = String::from_utf8(bytes.to_vec()).unwrap(); // .expect("Could not convert to UTF-8");

        let secret_key: SecretKey;
        let secret_key_op = SecretKey::from_str(secret_key_raw.as_str());
        match secret_key_op {
            Err(e) => {
                return Err(e.into());
            }
            Ok(val) => secret_key = val,
        }

        Ok(secret_key)
    }
    */
}

#[async_trait]
impl NFTsRepository for GanacheRepo {
    fn contract_id(&self) -> u16 {
        self.contract_id
    }
    async fn add(
        &self,
        asset_id: &Uuid,
        user_key: &KeyPair,
        hash_file: &String,
        prc: &u64,
        _cntr: &u64,
    ) -> ResultE<BlockchainTx> {
        let transport = web3::transports::Http::new(self.url.as_str()).unwrap();
        let web3 = web3::Web3::new(transport);

        let to: H160;
        let to_op = Address::from_str(user_key.address().as_str());
        match to_op {
            Err(e) => {
                return Err(NftUserAddressMalformedError(e.to_string()).into());
            }
            Ok(addr) => {
                to = addr.to_owned();
            }
        }

        let token = asset_id.to_string();
        let price = U256::from_dec_str((*prc).to_string().as_str()).unwrap();
        //let counter = U256::from_dec_str((*cntr).to_string().as_str()).unwrap();

        let contract_op = Contract::from_json(
            web3.eth(),
            self.contract_address.clone(),
            include_bytes!("../../res/LightNFT.abi"),
        );
        let contract = match contract_op {
            Err(e) => {
                return Err(AssetBlockachainError(e.to_string()).into());
            }
            Ok(cnt) => cnt,
        };

        //Estimate gas to consume
        let estimate_call = contract.estimate_gas(
            CONTRACT_METHOD_MINTING,
            (to.clone(), token.clone(), hash_file.clone(), price.clone()),
            self.contract_owner_address.clone(), //self.contract_address.clone(), //account_owner,
            Options::default(),
        );

        let estimate_op = estimate_call.await;

        let cost_gas: U256 = match estimate_op {
            Err(e) => {
                return Err(AssetBlockachainError(e.to_string()).into());
            }
            Ok(gas) => gas,
        };
        //request current gas price status
        let gas_price_op = web3.eth().gas_price().await;
        let gas_price: U256 = match gas_price_op {
            Err(e) => {
                return Err(AssetBlockachainError(e.to_string()).into());
            }
            Ok(gas) => gas,
        };

        //let nonce = U256::from(1); //
        //let nonce = generate_nonce(&self.counter )?;
        // debug!("nonce calculated: {}", nonce);

        let counter = web3
            .eth()
            .transaction_count(self.contract_address, None)
            .await
            .unwrap();
        debug!("{}", counter);
        //counter += U256::from_str_radix("1", 10).unwrap();
        //debug!("{}", counter);

        let tx_options = Options {
            gas: Some(cost_gas), // Some(U256::from_str("400000").unwrap()), //250.000 weis  30.000.000 //with 400.000 gas units works!
            gas_price: Some(gas_price), // None,     // Some(U256::from_str("10000000").unwrap()), //10.000.000 weis
            value: None,
            condition: None,
            transaction_type: None,
            nonce: None, //Some(counter), // None
            access_list: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
        };
        debug!("calling from {}", self.contract_address.to_string());

        let contract_owner_private_key;
        //let contract_owner_private_key_op = self.decrypt_contract_owner_secret_key().await;
        let contract_owner_private_key_op = SecretKey::from_str(
            restore_secret_key(self.contract_owner_secret.to_owned(), &self.kms_key_id, &self.config)
                .await
                .unwrap().as_str(),
        );

        match contract_owner_private_key_op {
            Err(_) => {
                return Err(HydrateMasterSecretKeyError {}.into());
            }
            Ok(value) => contract_owner_private_key = value,
        }

        let caller = contract.signed_call_with_confirmations(
            CONTRACT_METHOD_MINTING,
            (to.clone(), token.clone(), hash_file.clone(), price.clone()),
            tx_options,
            self.blockhain_node_confirmations.into(),
            &contract_owner_private_key,
        );

        let call_contract_op = caller.await;
        let tx = match call_contract_op {
            Err(e) => {
                return Err(AssetBlockachainError(format!("{:?}", e)).into());
            }
            Ok(transact) => transact,
        };
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
        let tx_paylaod = BlockchainTx::new(
            asset_id.to_owned(),
            Utc::now(),
            Some(tx.transaction_hash),
            tx.block_number,
            tx.gas_used,
            tx.effective_gas_price,
            Some(wei_to_gwei(tx.gas_used.unwrap()) * wei_to_gwei(tx.effective_gas_price.unwrap())),
            Some("gweis".to_string()),
            Some(tx.from),
            tx.to,
            self.contract_id,
            None,
        );
        Ok(tx_paylaod)
    }

    async fn get(&self, asset_id: &Uuid) -> ResultE<GanacheContentInfo> {
        let token = asset_id.to_string();

        let transport = web3::transports::Http::new(self.url.as_str()).unwrap();
        let web3 = web3::Web3::new(transport);

        //let contract_address = addr.clone();
        let contract_op = Contract::from_json(
            web3.eth(),
            self.contract_address.clone(),
            include_bytes!("../../res/LightNFT.abi"),
        );
        let contract = match contract_op {
            Err(e) => {
                return Err(AssetBlockachainError(e.to_string()).into());
            }
            Ok(cnt) => cnt,
        };

        let caller = contract.query(
            CONTRACT_METHOD_GET_CONTENT_BY_TOKEN,
            (token.clone(),),
            self.contract_address,
            Options::default(),
            None,
        );
        let call_contract_op: Result<GanacheContentInfo, web3::contract::Error> = caller.await;
        let res = match call_contract_op {
            Err(e) => {
                return Err(AssetBlockachainError(e.to_string()).into());
            }
            Ok(cnt) => cnt,
        };
        Ok(res)
    }
}

fn _wei_to_eth(wei_val: U256) -> f64 {
    let res = wei_val.as_u128() as f64;
    res / 1_000_000_000_000_000_000.0
}
fn wei_to_gwei(wei_val: U256) -> f64 {
    let res = wei_val.as_u128() as f64;
    res / 1_000_000_000.0
}

pub async fn block_status(client: &Web3<Http>) -> Block<H256> {
    let latest_block = client
        .eth()
        .block(BlockId::Number(BlockNumber::Latest))
        .await
        .unwrap()
        .unwrap();

    let timestamp = latest_block.timestamp.as_u64() as i64;
    let naive = NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
    let utc_dt: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    debug!(
            "[{}] block num {}, parent {}, transactions: {}, gas used {}, gas limit {}, base fee {}, difficulty {}, total difficulty {}",
            utc_dt.format("%Y-%m-%d %H:%M:%S"),
            latest_block.number.unwrap(),
            latest_block.parent_hash,
            latest_block.transactions.len(),
            latest_block.gas_used,
            latest_block.gas_limit,
            latest_block.base_fee_per_gas.unwrap(),
            latest_block.difficulty,
            latest_block.total_difficulty.unwrap()
        );
    return latest_block;
}
#[derive(Clone, Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct GanacheContentInfo {
    //field names coming from Solidity
    pub hashFile: String,
    pub uri: String,
    pub price: u64,
    pub state: ContentState,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ContentState {
    Active,
    Inactive,
}
impl fmt::Debug for ContentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Inactive => write!(f, "Inactive"),
        }
    }
}

impl fmt::Display for ContentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Inactive => write!(f, "Inactive"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseContentStateError;
impl std::str::FromStr for ContentState {
    type Err = ParseContentStateError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "Active" => Ok(ContentState::Active),
            "Inactive" => Ok(ContentState::Inactive),
            _ => Err(ParseContentStateError),
        }
    }
}

impl Detokenize for GanacheContentInfo {
    #[allow(non_snake_case)]
    fn from_tokens(tokens: Vec<web3::ethabi::Token>) -> Result<Self, web3::contract::Error>
    where
        Self: Sized,
    {
        let mut hashFile = "".to_string();
        let mut uri = "".to_string();
        let mut state = ContentState::Active;
        let mut price = 0;
        let mut i = 0;
        for token in tokens {
            debug!("{:?}", token);
            match i {
                0 => hashFile = token.into_string().unwrap(),
                1 => uri = token.into_string().unwrap(),
                2 => price = token.into_uint().unwrap().as_u64(),
                3 => state = ContentState::from_str(token.into_string().unwrap().as_str()).unwrap(),
                _ => {}
            };
            i += 1;
        }
        //let t = tokens[0].clone(); // .iter().next().unwrap().into_string().unwrap().clone();
        Ok(Self {
            hashFile,
            uri,
            price,
            state,
        })
    }
}

/*
fn generate_keypair() -> SecretKey {
    //let secp = secp256k1::Secp256k1::new();
    //let mut rng = rngs::StdRng::seed_from_u64(111); //TODO!!!! ojo!!!!!!!!!
    //secp.generate_keypair(&mut rng)


    let secp = Secp256k1::new();
    let secret_key = SecretKey::new(&mut rand::thread_rng());
    return secret_key;
}*/

fn _generate_nonce(counter: &Arc<Mutex<i32>>) -> Result<U256, NftBlockChainNonceMalformedError> {
    //generate unique nonce number
    let machine_id: i32;
    match get_mac_address() {
        Ok(Some(ma)) => {
            debug!("MAC addr = {}", ma);
            debug!("bytes = {:?}", ma.bytes());
            let original = ma.bytes();
            let original2 = [original[0], original[1], original[2], original[3]];
            let num = i32::from_be_bytes(original2);
            machine_id = num;
        }
        Ok(None) => {
            return Err(NftBlockChainNonceMalformedError(
                "no mac address found to choose the nodeId".to_string(),
            )
            .into())
        } // println!("No MAC address found."),
        Err(e) => return Err(NftBlockChainNonceMalformedError(e.to_string()).into()),
    }
    let counter_id: i32;
    {
        let data = Arc::clone(counter);
        let mut cont = data.lock().unwrap();
        counter_id = (*cont).clone();
        *cont += 1;
    }
    let mut id_generator_generator = SnowflakeIdGenerator::new(machine_id, counter_id);
    let id = id_generator_generator.real_time_generate();
    let nonce = U256::from(id);

    Ok(nonce)
}
