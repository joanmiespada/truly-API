use async_trait::async_trait;
use ethers::prelude::k256::elliptic_curve::bigint::ByteArray;
use lib_config::Config;
use log::{debug, trace};
use serde::{Serialize, Deserialize};
use std::{str::FromStr, fmt};
use uuid::Uuid;

use chrono::{format::format, DateTime, NaiveDateTime, Utc};

use web3::{
    contract::{Contract, Options, tokens::Detokenize},
    ethabi::Address,
    transports::Http,
    types::{Block, BlockId, BlockNumber, H256, U256},
    Web3,
};

const CONTRACT_METHOD_MINTING: &'static str = "mint";
const CONTRACT_METHOD_GET_CONTENT_BY_TOKEN: &'static str = "getContentByToken";

use crate::errors::asset::AssetBlockachainError;


type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error +Sync + Send >>;
#[async_trait]
pub trait NFTsRepository {
    //async
    async fn add(
        &self,
        asset_id: &Uuid,
        user_address: &String,
        hash_file: &String,
        price: &u64,
    ) -> ResultE<String>;
    async fn get(&self, asset_id: &Uuid) -> ResultE<GanacheContentInfo>;
}

#[derive(Clone, Debug)]
pub struct GanacheRepo {
    //web3: Web3<Http>,
    url: String,
    contract_address: Address,
    contract_owner: Address,
}

impl GanacheRepo {
    pub fn new(conf: &Config) -> GanacheRepo {
        let mut aux = conf.env_vars().contract_address();
        let contract_address_position = Address::from_str(aux.as_str()).unwrap();

        aux = conf.env_vars().contract_owner();
        let contract_owner_position = Address::from_str(aux.as_str()).unwrap();

        GanacheRepo {
            //web3: gateway,
            url: conf.env_vars().blockchain_url().to_owned(),
            contract_address: contract_address_position,
            contract_owner: contract_owner_position,
        }
    }
}

#[async_trait]
impl NFTsRepository for GanacheRepo {
    async fn add(
        &self,
        asset_id: &Uuid,
        user_address: &String,
        hash_file: &String,
        prc: &u64,
    ) -> ResultE<String> {
        let transport = web3::transports::Http::new(self.url.as_str()).unwrap();
        let web3 = web3::Web3::new(transport);

        let to = Address::from_str(user_address.as_str()).unwrap();
        let token = asset_id.to_string();
        let price = U256::from_dec_str((*prc).to_string().as_str()).unwrap();

        let price22 = price.as_u64();


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
        let estimate_call =
            contract.estimate_gas(
                CONTRACT_METHOD_MINTING, 
                (to.clone(), token.clone(), hash_file.clone(), price.clone()),
                self.contract_owner.clone(), //self.contract_address.clone(), //account_owner,
                Options::default());

        let estimate_op = estimate_call.await;

        let cost_gas:U256 = match estimate_op {
            Err(e) => {
                return Err(AssetBlockachainError(e.to_string()).into());
            }
            Ok(gas) => gas,
        };

        //30000000,
        //236197632
        let tx_options = Options {
            gas: Some(cost_gas),// Some(U256::from_str("400000").unwrap()), //250.000 weis  30.000.000 //with 400.000 gas units works!
            gas_price: None, // Some(U256::from_str("10000000").unwrap()), //10.000.000 weis
            value: None,
            condition: None,
            transaction_type: None,
            nonce: None,
            access_list: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
        };
        //block_status(&web3).await;
        debug!("calling from {}", self.contract_address.to_string());
        let caller = contract.call(
            CONTRACT_METHOD_MINTING,
            (to.clone(), token.clone(), hash_file.clone(), price.clone()),
            self.contract_owner.clone(), //self.contract_address.clone(), //account_owner,
            //Options::default(),
            tx_options,
        );
        let call_contract_op = caller.await;
        let tx = match call_contract_op {
            Err(e) => {
                return Err(AssetBlockachainError(e.to_string()).into());
            }
            Ok(transact) => transact,
        };
        let tx_str = format!("blockchain: ganache | tx: {:?}", tx);
        Ok(tx_str)
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
            self.contract_address, //to, //account_owner, //None
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

pub async fn block_status(client: &Web3<Http>) -> Block<H256> {
    let latest_block = client
        .eth()
        .block(BlockId::Number(BlockNumber::Latest))
        .await
        .unwrap()
        .unwrap();

    let timestamp = latest_block.timestamp.as_u64() as i64;
    let naive = NaiveDateTime::from_timestamp(timestamp, 0);
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
pub struct GanacheContentInfo {
    pub hashFile: String,
    pub uri: String,
    pub price: u64,
    pub state: ContentState,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ContentState {
    Active,
    Inactive
}
impl fmt::Debug for ContentState{
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
    fn from_tokens(tokens: Vec<web3::ethabi::Token>) -> Result<Self, web3::contract::Error>
    where
        Self: Sized {

        let mut hashFile= "".to_string();
        let mut uri ="".to_string();
        let mut state= ContentState::Active;
        let mut price = 0;
        let mut i=0;
        for token in tokens{
            debug!("{:?}", token);
            match i {
               0 => hashFile = token.into_string().unwrap(),
               1 => uri =  token.into_string().unwrap(),
               2 => price = token.into_uint().unwrap().as_u64(),
               3 => state = ContentState::from_str( token.into_string().unwrap().as_str()).unwrap() ,
               _ => {}
            }
            ;
            i+=1;
        }    
        //let t = tokens[0].clone(); // .iter().next().unwrap().into_string().unwrap().clone();
        Ok(Self{ 
            hashFile,
            uri,
            price,
            state
        })
    }
}