use std::{env, str::FromStr, time::Duration};

use chrono::{DateTime, NaiveDateTime, Utc};
//use ethers::{utils::{Ganache, parse_ether}, signers::{LocalWallet, Signer}, providers::{Provider, Middleware}, types::U256};
use hex::ToHex;
use lib_config::Config;
use serde::{Deserialize, Serialize};
use spectral::{assert_that, result::ResultAssertions};
//use hex_literal::hex;

use uuid::Uuid;
use web3::{
    contract::{tokens::Tokenizable, Contract, Options},
    transports::Http,
    types::{Address, Block, BlockId, BlockNumber, H160, H256, U256},
    Error, Web3,
};

/*
#[tokio::test]
async fn call_contract() {


    let mnemonic = "myth like bonus scare over problem client lizard pioneer submit female collect"; //from $ganache --deterministic command
    let ganache = Ganache::new().mnemonic(mnemonic).spawn();
    let wallet: LocalWallet = ganache.keys()[0].clone().into();
    let wallet_address: String = wallet.address().encode_hex();

    let provider = Provider::try_from(ganache.endpoint()).unwrap().interval(Duration::from_millis(10));
    let first_balance_op = provider.get_balance(wallet_address, None).await;

    assert_that!(&first_balance_op).is_ok();

    let first_balance = first_balance_op.unwrap();

    //let aux = first_balance.low_u64();
    let eth = U256::from(first_balance);
    assert_eq!(eth, parse_ether(1000).unwrap());

}*/

#[tokio::test]
async fn check_addresses() -> web3::Result<()> {
    env::set_var("ENVIRONMENT", "development");
    let mut config = Config::new();
    config.setup().await;

    let transport = web3::transports::Http::new(config.env_vars().blockchain_url())?;
    let web3 = web3::Web3::new(transport);

    let mut accounts = web3.eth().accounts().await?;
    for account in accounts {
        let balance_op = web3.eth().balance(account, None).await;
        assert_that!(&balance_op).is_ok();
        let balance = balance_op.unwrap();

        println!("Balance of {:?}: {}", account, balance);
    }
    Ok(())
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct GetContent {
    hashFile: String,
    uri: String,
    price: U256,
    state: String,
}

/* impl Tokenizable for GetContent {
    fn from_token(token: web3::ethabi::Token) -> Result<Self, web3::contract::Error>
    where
        Self: Sized,
    {
        todo!()
    }

    fn into_token(self) -> web3::ethabi::Token {
        todo!()
    }
} */

async fn block_status(client: Web3<Http>) -> Block<H256> {
    let latest_block = client
        .eth()
        .block(BlockId::Number(BlockNumber::Latest))
        .await
        .unwrap()
        .unwrap();

    let timestamp = latest_block.timestamp.as_u64() as i64;
    let naive = NaiveDateTime::from_timestamp(timestamp, 0);
    let utc_dt: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    println!(
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

#[tokio::test]
async fn call_contract2() -> web3::Result<()> {
    env::set_var("ENVIRONMENT", "development");
    let mut config = Config::new();
    config.setup().await;

    let transport = web3::transports::Http::new(config.env_vars().blockchain_url())?;
    let web3 = web3::Web3::new(transport);

    block_status(web3.clone()).await;

    let accounts = web3.eth().accounts().await?;
    let account_owner = accounts[0];
    let to = accounts[9];

    //let addr = config.env_vars().contract_address();
    let addr = Address::from_str("0x2612Af3A521c2df9EAF28422Ca335b04AdF3ac66").unwrap();  // address to LightNFT contract
    let token = Uuid::new_v4().to_string();
    let hashFile = "hash1234".to_string();
    let price = U256::from_str("2000").unwrap();

    let contract_address = addr.clone();
    let contract_op = Contract::from_json(
        web3.eth(),
        contract_address,
        include_bytes!("./LightNFT.abi"),
    );
    assert_that!(&contract_op).is_ok();

    let contract = contract_op.unwrap();

    /*
        let estimate_op = contract.estimate_gas(
            "mint",
            (to.clone(), token.clone(), hashFile.clone(), price.clone()),
            account_owner,
            Options::default(),
        );

        let estimate = estimate_op.await;

        assert_that!(&estimate).is_ok();
    */
    let tx_options = Options {
        gas: Some(U256::from_str("250000").unwrap()), //1.000.000 weis
        gas_price: Some(U256::from_str("10000000").unwrap()), //100 weis
        value: None,
        condition: None,
        transaction_type: None,
        nonce: None,
        access_list: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };

    let caller_op = contract.call(
        "mint",
        (to.clone(), token.clone(), hashFile.clone(), price.clone()),
        account_owner,
       //Options::default(), 
        tx_options, 
    );
    //let call_contract_op : Result<bool ,web3::contract::Error>= caller_op.await;
    let call_contract_op = caller_op.await;

    assert_that!(&call_contract_op).is_ok();

    let tx = call_contract_op.unwrap();

    println!("TxHash: {}", tx);
    
    block_status(web3.clone()).await;

    std::thread::sleep(std::time::Duration::from_secs(2));


    let caller_op2 = contract.query(
        "getContentByToken",
        (token.clone(),),
        to, //account_owner, //None
        Options::default(),
        None,
    );
    let call_contract_op2: Result<String, web3::contract::Error> = caller_op2.await;

    assert_that!(&call_contract_op2).is_ok();
    let values_in_chain = call_contract_op2.unwrap();
    assert_eq!(values_in_chain, hashFile);

    Ok(())
}

#[tokio::test]
async fn call_contract3() -> web3::Result<()> {
    env::set_var("ENVIRONMENT", "development");
    let mut config = Config::new();
    config.setup().await;

    let transport = web3::transports::Http::new(config.env_vars().blockchain_url())?;
    let web3 = web3::Web3::new(transport);
    
    block_status(web3.clone()).await;

    let accounts = web3.eth().accounts().await?;
    let account_owner = accounts[0];
    let account_creator = accounts[9];

    //let addr = config.env_vars().contract_address();
    let addr = Address::from_str("0xA57B8a5584442B467b4689F1144D269d096A3daF").unwrap();  // address to SimpleTest contract

    //let contract_address = web3::types::H160::from_str(addr.as_str()).unwrap();
    let contract_op = Contract::from_json(
        web3.eth(),
        addr, //contract_address,
        include_bytes!("./SimpleTest.abi"),
    );
    assert_that!(&contract_op).is_ok();

    let contract = contract_op.unwrap();

    let value = U256::from_str("24").unwrap();
    
    let tx_options = Options {
        gas: Some(U256::from_str("25000").unwrap()), //1.000.000 weis
        gas_price: Some(U256::from_str("10000000").unwrap()), //100 weis
        value: None,
        condition: None,
        transaction_type: None,
        nonce: None,
        access_list: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };

    let caller_op1 = contract.call(
        "set",
        (value,),        //(22_u32,),//value,//(account_creator, token, hash),
        account_creator, //account_owner,
        tx_options,  
        //Options::default(),
        //None,
    );
    let call_contract_op1 = caller_op1.await;

    assert_that!(&call_contract_op1).is_ok();

    let tx = call_contract_op1.unwrap();

    println!("TxHash: {}", tx);

    block_status(web3.clone()).await;
    std::thread::sleep(std::time::Duration::from_secs(1));

    let caller_op2 = contract.query(
        "get",
        (),            //(account_creator, token, hash),
        account_owner, //None
        Options::default(),
        None,
    );
    let call_contract_op2: Result<U256, web3::contract::Error> = caller_op2.await;

    assert_that!(&call_contract_op2).is_ok();

    assert_eq!(call_contract_op2.unwrap(), value);

    Ok(())
}
