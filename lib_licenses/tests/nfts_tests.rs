use crate::build_dynamodb;
use aws_sdk_dynamodb::Client;
use ethers::utils::Ganache;
use lib_config::Config;
use lib_licenses::repositories::assets::AssetRepo;
use lib_licenses::repositories::owners::OwnerRepo;
use lib_licenses::repositories::schema_owners::create_schema_owners;
use lib_licenses::services::assets::{AssetManipulation, AssetService};
use lib_licenses::services::owners::OwnerService;
use lib_licenses::{models::asset::Asset, repositories::schema_asset::create_schema_assets};
use lib_licenses::{
    repositories::ganache::{block_status, GanacheRepo},
    services::nfts::{NFTsManipulation, NFTsService, NTFState},
};
use spectral::{assert_that, result::ResultAssertions};
use std::time::Duration;
use std::{env, str::FromStr};
use testcontainers::*;
use url::Url;
use web3::{
    contract::{Contract, Options},
    types::{H160, U256},
};

const MNEMONIC_TEST: &str =
    "myth like bonus scare over problem client lizard pioneer submit female collect"; //from $ganache --deterministic command

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

#[tokio::test]
async fn create_contract_and_mint_nft_test() -> web3::Result<()> {
    env::set_var("RUST_LOG", "debug");
    env::set_var("ENVIRONMENT", "development");

    env_logger::builder().is_test(true).init();

    let mut config = Config::new();
    config.setup().await;

    //create dynamodb
    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config = build_dynamodb(host_port).await;
    let client = Client::new(&shared_config);

    let creation1 = create_schema_assets(&client).await;
    assert_that(&creation1).is_ok();

    let creation2 = create_schema_owners(&client).await;
    assert_that(&creation2).is_ok();

    config.set_aws_config(&shared_config);

    //create fake asset

    let repo_ow = OwnerRepo::new(&config);
    let owner_service = OwnerService::new(repo_ow);

    let repo_as = AssetRepo::new(&config);
    let asset_service = AssetService::new(repo_as);

    let asset_url: Url = Url::parse("http://www.file1.com/test1.mp4").unwrap();
    let asset_hash: String = "hash1234".to_string();
    let asset_license: String =
        String::from_str("license - open shared social networks - forbiden mass media").unwrap();

    let mut as1 = Asset::new();
    as1.set_url(&Some(asset_url));
    as1.set_hash(&Some(asset_hash));
    as1.set_license(&Some(asset_license));

    let user_id = String::from_str("user1234-1234-1234-1234").unwrap();

    let new_asset_op = asset_service.add(&mut as1, &user_id).await;

    assert_that!(&new_asset_op).is_ok();

    //create contract and deploy

    let ganache = Ganache::new().mnemonic(MNEMONIC_TEST).spawn();
    let url = ganache.endpoint();

    let http = web3::transports::Http::new(url.as_str())?;
    let web3 = web3::Web3::new(http);
    let accounts_op = web3.eth().accounts().await;
    let user_account = format!("{:?}", accounts_op.clone().unwrap()[9]);
    let contract_owner_account = format!("{:?}", accounts_op.clone().unwrap()[0]);

    let bytecode = include_str!("../res/LightNFT.bin").trim_end();

    let contract_deploy_op = Contract::deploy(web3.eth(), include_bytes!("../res/LightNFT.abi"))
        .unwrap()
        .confirmations(0)
        .poll_interval(Duration::from_secs(10))
        //.options(Options::default())
        .options(Options::with(|opt| {
            //    opt.value       = Some(U256::from_str("1").unwrap()); //Some(0.into());
            //opt.gas_price   = Some(U256::from_str("2000000000").unwrap());
            opt.gas = Some(U256::from_str("1000000").unwrap());
        }))
        .execute(
            bytecode,
            (),
            H160::from_str(contract_owner_account.as_str()).unwrap(),
        )
        .await;

    assert_that!(&contract_deploy_op).is_ok();

    let contract_address = format!("{:?}", contract_deploy_op.unwrap().address());

    let mut new_configuration = config.env_vars().clone();
    new_configuration.set_blockchain_url(url);
    new_configuration.set_contract_address(contract_address);
    new_configuration.set_contract_owner(contract_owner_account);
    config.set_env_vars(&new_configuration);

    let repo = GanacheRepo::new(&config);
    let nft_service = NFTsService::new(repo, asset_service.clone(), owner_service.clone());

    let price: u64 = 2000;

    let mint_op = nft_service
        .add(as1.id(), &user_id, &user_account.to_string(), &price)
        .await;

    assert_that!(&mint_op).is_ok();
    let tx_in_chain = mint_op.unwrap();

    let check_op = nft_service.get(as1.id()).await;

    assert_that!(&check_op).is_ok();

    let content = check_op.unwrap();

    assert_eq!(content.hash_file, as1.hash().as_deref().unwrap());
    assert_eq!(content.price, price);
    assert_eq!(content.state, NTFState::Active);

    let tx_op = asset_service.get_by_id(as1.id()).await;

    assert_that!(&tx_op).is_ok();

    let content_minted = tx_op.unwrap();

    assert_eq!(tx_in_chain, content_minted.minted_tx().as_deref().unwrap());

    Ok(())
}

#[tokio::test]
async fn create_simple_contract_test() -> web3::Result<()> {
    env::set_var("RUST_LOG", "debug");
    env::set_var("ENVIRONMENT", "development");
    env_logger::builder().is_test(true).init();
    let mut config = Config::new();
    config.setup().await;

    let ganache = Ganache::new().mnemonic(MNEMONIC_TEST).spawn();
    let url = ganache.endpoint();

    let transport = web3::transports::Http::new(url.as_str())?;
    let web3 = web3::Web3::new(transport);

    let accounts_op = web3.eth().accounts().await;
    //let user_account = format!("{:?}", accounts_op.clone().unwrap()[9]);
    let contract_owner_account_str = format!("{:?}", accounts_op.clone().unwrap()[0]);
    let contract_owner = H160::from_str(contract_owner_account_str.as_str()).unwrap();

    let bytecode = include_str!("../res/SimpleTest.bin").trim_end();

    let contract_deploy_op = Contract::deploy(web3.eth(), include_bytes!("../res/SimpleTest.abi"))
        .unwrap()
        .confirmations(0)
        .poll_interval(Duration::from_secs(10))
        //.options(Options::default())
        .options(Options::with(|opt| {
            //    opt.value       = Some(U256::from_str("1").unwrap()); //Some(0.into());
            //opt.gas_price   = Some(U256::from_str("2000000000").unwrap());
            opt.gas = Some(U256::from_str("1000000").unwrap());
        }))
        .execute(bytecode, (), contract_owner)
        .await;

    assert_that!(&contract_deploy_op).is_ok();

    let contract_address_str = format!("{:?}", contract_deploy_op.unwrap().address());

    //block_status(&web3).await;

    let contract_address = H160::from_str(contract_address_str.as_str()).unwrap();

    //let contract_address = web3::types::H160::from_str(addr.as_str()).unwrap();
    let contract_op = Contract::from_json(
        web3.eth(),
        contract_address,
        include_bytes!("../res/SimpleTest.abi"),
    );
    assert_that!(&contract_op).is_ok();

    let contract = contract_op.unwrap();

    let value = U256::from_str("24").unwrap();

    let estimate_call = contract.estimate_gas("set", value, contract_owner, Options::default());

    let estimate_op = estimate_call.await;

    assert_that!(&estimate_op).is_ok();

    let cost_gas: U256 = estimate_op.unwrap().into();

    let tx_options = Options {
        gas: Some(cost_gas), //Some(U256::from_str("400000").unwrap()), //1.000.000 weis
        gas_price: None,     // Some(U256::from_str("10000000").unwrap()), //100 weis
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
        (value,),       //(22_u32,),//value,//(account_creator, token, hash),
        contract_owner, //account_creator, //account_owner,
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
        (),             //(account_creator, token, hash),
        contract_owner, // account_owner, //None
        Options::default(),
        None,
    );
    let call_contract_op2: Result<U256, web3::contract::Error> = caller_op2.await;

    assert_that!(&call_contract_op2).is_ok();

    assert_eq!(call_contract_op2.unwrap(), value);

    Ok(())
}

fn _wei_to_eth(wei_val: U256) -> f64 {
    let res = wei_val.as_u128() as f64;
    res / 1_000_000_000_000_000_000.0
}
