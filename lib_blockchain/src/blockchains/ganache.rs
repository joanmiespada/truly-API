use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use lib_config::{
    config::Config,
    environment::DEV_ENV,
    infra::{restore_secret_key, store_secret_key},
};
use log::debug;
use secp256k1::SecretKey;
use std::str::FromStr;
use url::Url;
use uuid::Uuid;

use web3::{
    contract::{tokens::Detokenize, Contract, Options},
    transports::Http,
    types::{Address, Block, BlockId, BlockNumber, H160, H256, U256},
    Web3, //, signing::SecretKey,
};

use crate::{
    errors::nft::HydrateMasterSecretKeyError,
    models::block_tx::BlockchainTx,
    repositories::{
        blockchain::BlockchainRepo, blockchain::BlockchainRepository, contract::ContractRepo,
        contract::ContractRepository,
    },
};
use crate::{errors::nft::NftUserAddressMalformedError, models::keypair::KeyPair};

const CONTRACT_METHOD_MINTING: &'static str = "mint";
const CONTRACT_METHOD_GET_CONTENT_BY_TOKEN: &'static str = "getContentByToken";

use lib_licenses::errors::asset::AssetBlockachainError;

use super::chain::{ContentState, ContractContentInfo, NFTsRepository};

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[derive(Clone, Debug)]
pub struct GanacheBlockChain {
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

impl GanacheBlockChain {
    pub async fn new(
        conf: &Config,
        contracts_repo: &ContractRepo,
        blockchains_repo: &BlockchainRepo,
    ) -> ResultE<GanacheBlockChain> {
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

        let contract_address =
            H160::from_str(contract.address().clone().unwrap().as_str()).unwrap();
        let contract_owner_address =
            H160::from_str(contract.owner_address().clone().unwrap().as_str()).unwrap();

        Ok(GanacheBlockChain {
            url: blockchain_url.to_owned(),
            contract_address,       //contract_address_position,
            contract_owner_address, //contract_owner_position,
            contract_owner_secret: contract.owner_secret().clone().unwrap().to_owned(), //contract_owner_position,
            kms_key_id: conf.env_vars().kms_key_id().unwrap(),
            //aws: conf.aws_config().to_owned(),
            config: conf.clone(),
            blockhain_node_confirmations: blockchain.confirmations().to_owned(), //conf.env_vars().blockchain_confirmations().to_owned(),
            contract_id: aux.to_owned(), //contract.to_owned(),
        })
    }
}

#[async_trait]
impl NFTsRepository for GanacheBlockChain {
    fn contract_id(&self) -> u16 {
        self.contract_id
    }
    async fn add(
        &self,
        asset_id: &Uuid,
        user_key: &KeyPair,
        hash_file: &String,
        _hash_algorithm: &String,
        prc: &Option<u64>,
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
        let price = U256::from_dec_str((prc.unwrap()).to_string().as_str()).unwrap();
        //let counter = U256::from_dec_str((*cntr).to_string().as_str()).unwrap();

        let contract_op = Contract::from_json(
            web3.eth(),
            self.contract_address.clone(),
            include_bytes!("../../res/evm/LightNFT.abi"),
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
            restore_secret_key(
                self.contract_owner_secret.to_owned(),
                &self.kms_key_id,
                &self.config,
            )
            .await
            .unwrap()
            .as_str(),
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
            Some(tx.transaction_hash.to_string()),
            Some(tx.block_number.unwrap().as_u64()),
            Some(tx.gas_used.unwrap().to_string()),
            Some(tx.effective_gas_price.unwrap().to_string()),
            Some(
                wei_to_gwei(tx.gas_used.unwrap())
                    * wei_to_gwei(tx.effective_gas_price.unwrap_or_default()),
            ),
            Some("gweis".to_string()),
            Some(tx.from.to_string()),
            Some(tx.to.unwrap().to_string()),
            self.contract_id,
            None,
        );
        Ok(tx_paylaod)
    }

    async fn get(&self, asset_id: &String) -> ResultE<ContractContentInfo> {
        let token = asset_id.clone();

        let transport = web3::transports::Http::new(self.url.as_str()).unwrap();
        let web3 = web3::Web3::new(transport);

        //let contract_address = addr.clone();
        let contract_op = Contract::from_json(
            web3.eth(),
            self.contract_address.clone(),
            include_bytes!("../../res/evm/LightNFT.abi"),
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
        let call_contract_op: Result<ContractContentInfo, web3::contract::Error> = caller.await;
        if let Err(e) = call_contract_op {
            return Err(AssetBlockachainError(e.to_string()).into());
        }
        let mut cnt = call_contract_op.ok().unwrap();
        cnt.token = Some(asset_id.to_string());

        Ok(cnt)
    }

    async fn create_keypair(&self, user_id: &String) -> ResultE<(KeyPair, bool)> {
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

        let user_private_key_cyphered =
            store_secret_key(&user_private_key, &self.kms_key_id, &self.config).await?;
        let user_public_key_cyphered =
            store_secret_key(&user_public_key, &self.kms_key_id, &self.config).await?;

        let mut user_key = KeyPair::new();
        user_key.set_user_id(user_id);
        user_key.set_address(&user_address);
        user_key.set_private_key(&user_private_key_cyphered);
        user_key.set_public_key(&user_public_key_cyphered);

        Ok((user_key, true))
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
            latest_block.base_fee_per_gas.unwrap_or_default(),
            latest_block.difficulty,
            latest_block.total_difficulty.unwrap()
        );
    return latest_block;
}

impl Detokenize for ContractContentInfo {
    #[allow(non_snake_case)]
    fn from_tokens(tokens: Vec<web3::ethabi::Token>) -> Result<Self, web3::contract::Error>
    where
        Self: Sized,
    {
        let mut hashFile = "".to_string();
        let mut hashAlgo = "".to_string();
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
                4 => hashAlgo = token.into_string().unwrap(),
                _ => {}
            };
            i += 1;
        }
        //let t = tokens[0].clone(); // .iter().next().unwrap().into_string().unwrap().clone();
        Ok(Self {
            hashFile,
            hashAlgo,
            uri: Some(uri),
            price: Some(price),
            state: Some(state),
            token: None,
        })
    }
}

// impl CloneBoxNFTsRepository for GanacheBlockChain {
//     fn clone_box(&self) -> Box<dyn NFTsRepository> {
//         GanacheBlockChain {
//             url: self.url.clone(),
//             contract_address: self.contract_address.clone(),
//             contract_owner_address: self.contract_owner_address.clone(),
//             contract_owner_secret: self.contract_owner_secret.clone(),
//             kms_key_id: self.kms_key_id.clone(),
//             //aws: SdkConfig,
//             config: self.config.clone(),
//             blockhain_node_confirmations: self.blockhain_node_confirmations.clone(),
//             contract_id: self.contract_id.clone(),
//         }
//     }
// }

// impl CloneBoxNFTsRepository for GanacheBlockChain {
//     fn clone_box(&self) -> Box<dyn NFTsRepository> {
//         Box::new(self.clone())
//     }
//}
