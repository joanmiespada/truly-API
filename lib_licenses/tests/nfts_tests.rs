
use core::time;
use std::{env, str::FromStr, time::Duration};

use chrono::{DateTime, NaiveDateTime, Utc};
use ethers::{utils::{Ganache}};// signers::{LocalWallet, Signer}, providers::{Provider, Middleware}, types::U256};
use hex::ToHex;
use lib_config::Config;
use lib_licenses::{repositories::ganache::{block_status, GanacheRepo}, services::nfts::{NFTsService,NFTsManipulation } };
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

const MNEMONIC_TEST: &str = "myth like bonus scare over problem client lizard pioneer submit female collect"; //from $ganache --deterministic command

#[tokio::test]
async fn ganache_bootstrap_get_balance_test() {

    env::set_var("ENVIRONMENT", "development");
    let mut config = Config::new();
    config.setup().await;

    let ganache = Ganache::new().mnemonic(MNEMONIC_TEST).spawn();
    let url = ganache.endpoint();

    let transport = web3::transports::Http::new(url.as_str()).unwrap();
    let web3 = web3::Web3::new(transport);

    let accounts_op = web3.eth().accounts().await;
    assert_that!(&accounts_op).is_ok();
    let accounts = accounts_op.unwrap();
    let ibalance_op = web3.eth().balance(accounts[0], None).await;
    assert_that!(&ibalance_op).is_ok();
    drop(ganache)
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct GetContent {
    hashFile: String,
    uri: String,
    price: U256,
    state: String,
}


#[tokio::test]
async fn create_contract_and_mint_test() -> web3::Result<()> {

    env::set_var("RUST_LOG", "debug");
    env::set_var("ENVIRONMENT", "development");

    env_logger::builder().is_test(true).try_init();

    let mut config = Config::new();
    config.setup().await;
    
    let ganache = Ganache::new().mnemonic(MNEMONIC_TEST).spawn();
    let url = ganache.endpoint();

    let http = web3::transports::Http::new(url.as_str())?;
    let web3 = web3::Web3::new(http);
    let accounts_op = web3.eth().accounts().await;
    let user_account = accounts_op.clone().unwrap()[9];
    //let contract_owner_account =Address::from_str("0x90F8bf6A479f320ead074411a4B0e7944Ea8c9C1").unwrap();   // accounts_op.clone().unwrap()[0];
    let contract_owner_account = accounts_op.clone().unwrap()[0];

    let bytecode = include_str!("../res/LightNFT.bin").trim_end();

    let contract_deploy_op = Contract::deploy(web3.eth(), include_bytes!("../res/LightNFT.abi"))
        .unwrap()
        .confirmations(0)
        .poll_interval(time::Duration::from_secs(10))
        //.options(Options::default())
        .options(Options::with(|opt| {
        //    opt.value       = Some(U256::from_str("1").unwrap()); //Some(0.into());
            //opt.gas_price   = Some(U256::from_str("2000000000").unwrap());
            opt.gas         = Some(U256::from_str("1000000").unwrap());
        }))
        .execute(
            bytecode,
            (),
            contract_owner_account,
        )
        .await;

    assert_that!(&contract_deploy_op).is_ok();

    let contract_address = contract_deploy_op.unwrap().address().clone();

    let mut new_configuration = config.env_vars().clone();
    new_configuration.set_blockchain_url(url);
    new_configuration.set_contract_address(  contract_address.as_bytes() );
    new_configuration.set_contract_owner(contract_owner_account.to_string());
    config.set_env_vars(&new_configuration);

    let repo = GanacheRepo::new(&config);
    let service = NFTsService::new(repo);

    let token = Uuid::new_v4();
    let hash_file = "hash1234".to_string();
    let price:u64 = 2000; 
    
    let mint_op = service.add(&token, &user_account.to_string(), &hash_file, &price).await;

    assert_that!(&mint_op).is_ok();

    std::thread::sleep(std::time::Duration::from_secs(2));

    let check_op = service.get(&token).await;
    
    assert_that!(&check_op).is_ok();

    let hash_recovered = check_op.unwrap();

    assert_eq!( hash_recovered, hash_file );
    
    drop(ganache);
    Ok(())
}


#[tokio::test]
async fn mint_test() -> web3::Result<()> {

    env::set_var("RUST_LOG", "debug");
    env::set_var("ENVIRONMENT", "development");

    env_logger::builder().is_test(true).try_init();

    let mut config = Config::new();
    config.setup().await;
    
    let url = "http://127.0.0.1:8545".to_owned(); //ganache.endpoint();

    //let contract_owner_account =Address::from_str("0x90F8bf6A479f320ead074411a4B0e7944Ea8c9C1").unwrap();   // accounts_op.clone().unwrap()[0];
    let contract_owner_account = "0x90F8bf6A479f320ead074411a4B0e7944Ea8c9C1";   // accounts_op.clone().unwrap()[0];

    //let contract_address = Address::from_str("0xCfEB869F69431e42cdB54A4F4f105C19C080A601").unwrap();
    let contract_address = "0xCfEB869F69431e42cdB54A4F4f105C19C080A601";

    let mut new_configuration = config.env_vars().clone();
    new_configuration.set_blockchain_url(url);
    new_configuration.set_contract_address(contract_address.to_string());
    new_configuration.set_contract_owner(contract_owner_account.to_string());
    config.set_env_vars(&new_configuration);

    let repo = GanacheRepo::new(&config);
    let service = NFTsService::new(repo);

    let token = Uuid::new_v4();
    let hash_file = "hash1234".to_string();
    let price:u64 = 2000; 
    
    
    let mint_op = service.add(&token, &contract_owner_account.to_string(), &hash_file, &price).await;

    assert_that!(&mint_op).is_ok();

    std::thread::sleep(std::time::Duration::from_secs(2));

    let check_op = service.get(&token).await;
    
    assert_that!(&check_op).is_ok();

    let hash_recovered = check_op.unwrap();

    assert_eq!( hash_recovered, hash_file );

    Ok(())

    
}



#[tokio::test]
async fn simple_test() -> web3::Result<()> {
    env::set_var("ENVIRONMENT", "development");
    let mut config = Config::new();
    config.setup().await;

    let transport = web3::transports::Http::new(config.env_vars().blockchain_url())?;
    let web3 = web3::Web3::new(transport);
    
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
    block_status(&web3).await;

    let accounts = web3.eth().accounts().await?;
    let account_owner = accounts[0];
    let account_creator = accounts[9];

    //let addr = config.env_vars().contract_address();
    let addr = Address::from_str("0x254dffcd3277C0b1660F6d42EFbB754edaBAbC2B").unwrap();  // address to SimpleTest contract

    //let contract_address = web3::types::H160::from_str(addr.as_str()).unwrap();
    let contract_op = Contract::from_json(
        web3.eth(),
        addr, //contract_address,
        include_bytes!("../res/SimpleTest.abi"),
    );
    assert_that!(&contract_op).is_ok();

    let contract = contract_op.unwrap();

    let value = U256::from_str("24").unwrap();
    
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

    block_status(&web3).await;
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

fn wei_to_eth(wei_val: U256) -> f64 {
    let res = wei_val.as_u128() as f64;
    res / 1_000_000_000_000_000_000.0
}
