use std::{time::Duration, env, str::FromStr};

//use ethers::{utils::{Ganache, parse_ether}, signers::{LocalWallet, Signer}, providers::{Provider, Middleware}, types::U256};
use hex::ToHex;
use lib_config::Config;
use spectral::{assert_that, result::ResultAssertions};
//use hex_literal::hex;


use uuid::Uuid;
use web3::{
    contract::{Contract, Options},
    types::U256,
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
async fn call_contract2() -> web3::Result<()> {

    env::set_var("ENVIRONMENT", "development");
    let mut config = Config::new();
    config.setup().await;


    let transport = web3::transports::Http::new(config.env_vars().blockchain_url())?;
    let web3 = web3::Web3::new(transport);

    let mut accounts = web3.eth().accounts().await?;
    let account_owner = accounts[0];
    let account_creator = accounts[9];

    let addr = config.env_vars().contract_address();
    //let contract_address = hex!( addr.as_str() ).into();

    let contract_address =  web3::types::H160::from_str(addr.as_str() ).unwrap();// (()) a contract.address();
    let contract_op = Contract::from_json(
        web3.eth(),
        contract_address,
        include_bytes!("./LightNFT.json"),
    );
    assert_that!(&contract_op).is_ok();
    
    let contract = contract_op.unwrap();

    let token = Uuid::new_v4().to_string();
    let hash = "hash1234".to_string();
    let caller_op = contract.query("mint", (account_creator,token,hash), account_owner, Options::default(), None);
    let call_contract_op = caller_op.await;

    assert_that!(&call_contract_op ).is_ok();
    
    Ok(())
}
