use aws_sdk_kms::model::KeyUsageType;
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::signers::LocalWallet;
use ethers::utils::Ganache;
use ethers_solc::Solc;
use lib_config::{config::Config, secrets::SECRETS_MANAGER_KEYS, secrets::SECRETS_MANAGER_SECRET_KEY};
use lib_licenses::repositories::ganache::block_status;
use aws_sdk_kms::types::Blob;
use base64::{engine::general_purpose, Engine as _};
use spectral::{assert_that, result::ResultAssertions};
use std::time::Duration;
use std::{env, str::FromStr};
use std::{path::Path, sync::Arc};
use testcontainers::*;
use web3::{
    contract::{Contract, Options},
    types::{H160, U256},
};
use lib_config::infra::build_local_stack_connection;

pub const MNEMONIC_TEST: &str =
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
    let mut accounts = accounts_op.unwrap();
    accounts.push("00a329c0648769a73afac7f9381e08fb43dbea72".parse().unwrap());

    println!("Accounts: {:?}", accounts);
    for account in accounts {
        let balance = web3.eth().balance(account, None).await.unwrap();
        println!("Balance of {:?}: {}", account, balance);
    }
    //let ibalance_op = web3.eth().balance(accounts[0], None).await;
    //assert_that!(&ibalance_op).is_ok();

    //let mut wallet = web3.eth().accounts().await;

    drop(ganache)
}

pub async fn deploy_contract_web3(
    url: &str,
    contract_owner_address: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let http = web3::transports::Http::new(url)?;
    let web3 = web3::Web3::new(http);

    let gas_price = web3.eth().gas_price().await.unwrap();
    //let chain_id = web3.eth().chain_id().await.unwrap().as_u64();

    let bytecode = include_str!("../res/LightNFT.bin").trim_end();
    let abi = include_bytes!("../res/LightNFT.abi");

    let contract_deploy_op = Contract::deploy(web3.eth(), abi)
        .unwrap()
        .confirmations(0)
        .poll_interval(Duration::from_secs(10))
        //.options(Options::default())
        .options(Options::with(|opt| {
            //    opt.value       = Some(U256::from_str("1").unwrap()); //Some(0.into());
            //opt.gas_price   = Some(U256::from_str("2000000000").unwrap());
            opt.gas_price = Some(gas_price);
            opt.gas = Some(U256::from_str("1000000").unwrap()); //only execute: 1000000
        }))
        .execute(
            bytecode,
            (),
            H160::from_str(&contract_owner_address).unwrap(),
        )
        //.sign_with_key_and_execute(bytecode, (), &contract_owner_private, Some(chain_id))
        .await;

    assert_that!(&contract_deploy_op).is_ok();

    let contract_address = format!("{:?}", contract_deploy_op.unwrap().address());

    return Ok(contract_address);
}

pub async fn _deploy_contract_ethers(
    url: &str,
    wallet: &LocalWallet,
) -> Result<String, Box<dyn std::error::Error>> {
    type Client = SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>;

    //use std::fs::File;
    //use std::io::prelude::*;

    let provider = Provider::<Http>::try_from(url.clone())?;

    let ethers_client = SignerMiddleware::new(provider.clone(), wallet.clone());
    let file = format!("{}/res/LightNFT.sol", env!("CARGO_MANIFEST_DIR"));
    //let file = format!("../res/LightNFT.sol");
    let source = Path::new(&file);

    // let mut file_handler = File::open(source)?;
    // let mut content = String::new();
    // file_handler.read_to_string(&mut content)?;
    // drop(file_handler);

    let compiled = Solc::default().compile_source(source)?;
    //.expect("Could not compile contracts");
    // let compiled;
    // match compiled_op {
    //     Err(e) => { return Err(e.into()); },
    //     Ok(val)=> { compiled=val;}
    // }

    let (abi, bytecode, _runtime_bytecode) = compiled
        .find("LightNFT")
        //.unwrap()
        .expect("could not find contract")
        .into_parts_or_default();

    let factory = ContractFactory::new(abi, bytecode, Arc::new(ethers_client.clone()));

    let contract = factory.deploy(())?.send().await?;

    let addr = contract.address();

    let addr_string = format!("{:#?}", addr);
    return Ok(addr_string);
}

pub async fn store_secret_key(
    info_to_be_encrypted: &str,
    kms_key_id: &str,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let aws = config.aws_config();

    let client = aws_sdk_kms::Client::new(aws);

    let blob = Blob::new(info_to_be_encrypted.as_bytes());
    let resp_op = client
        .encrypt()
        .key_id(kms_key_id.clone())
        .plaintext(blob)
        .send()
        .await;
    let resp = resp_op.unwrap();

    let blob = resp.ciphertext_blob.expect("Could not get encrypted text");
    let bytes = blob.as_ref();

    //let value = base64::encode(bytes);
    let value = general_purpose::STANDARD.encode(bytes);

    let client2 = aws_sdk_secretsmanager::Client::new(aws);

    client2
        .put_secret_value()
        .secret_id(SECRETS_MANAGER_SECRET_KEY)
        .secret_string(value)
        .send()
        .await?;

    Ok(())
}

pub async fn restore_secret_key(
    kms_key_id: &str,
    config: &Config,
) -> Result<String, Box<dyn std::error::Error>> {
    let aws = config.aws_config();

    let client = aws_sdk_secretsmanager::Client::new(&aws);
    let scr = client
        .get_secret_value()
        .secret_id(SECRETS_MANAGER_SECRET_KEY)
        .send()
        .await?;

    let secret_key_cyphered = scr.secret_string().unwrap();

    let value = general_purpose::STANDARD
        .decode(secret_key_cyphered)
        .unwrap();

    let client2 = aws_sdk_kms::Client::new(&aws);

    let data = aws_sdk_kms::types::Blob::new(value);

    let resp;
    let resp_op = client2
        .decrypt()
        .key_id(kms_key_id.to_owned())
        .ciphertext_blob(data.to_owned())
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

    Ok(secret_key_raw)
}

    
pub async fn create_secrets(
    client: &aws_sdk_secretsmanager::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .create_secret()
        .name(SECRETS_MANAGER_SECRET_KEY.to_string())
        .secret_string(  "to be overwritten".to_string()  )
        .send()
        .await?;
    
    let secrets_json = r#"
    {
        "HMAC_SECRET" : "localtest_hmac_1234RGsdfg#$%",
        "JWT_TOKEN_BASE": "localtest_jwt_sd543ERGds235$%^"
    }
    "#;

    client
        .create_secret()
        .name(SECRETS_MANAGER_KEYS.to_string())
        .secret_string(  secrets_json  )
        .send()
        .await?;

    Ok(())
}

pub async fn create_key(client: &aws_sdk_kms::Client) -> Result<String, Box<dyn std::error::Error>> {
    let resp =client
        .create_key()
        .key_usage(KeyUsageType::EncryptDecrypt)
        .send()
        .await?;
    let id = resp
        .key_metadata
        .unwrap()
        .key_id
        .unwrap();
        //.unwrap_or_else(|| String::from("No ID!"));

    Ok(id)
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

#[tokio::test]
async fn set_up_secret() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "debug");
    env::set_var("ENVIRONMENT", "development");

    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();

    let mut local_stack = images::local_stack::LocalStack::default();
    local_stack.set_services("secretsmanager,kms");
    let node = docker.run(local_stack);
    let host_port = node.get_host_port_ipv4(4566);

    let shared_config = build_local_stack_connection(host_port).await;

    let mut config = Config::new();
    config.setup().await;
    config.set_aws_config(&shared_config);

    let keys_client = aws_sdk_kms::client::Client::new(&shared_config);
    let kms_id = create_key(&keys_client).await?;
    let secrets_client = aws_sdk_secretsmanager::client::Client::new(&shared_config);
    create_secrets(&secrets_client).await?;

    let secret: &str = "4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d"; // secret key example
    
    store_secret_key(secret, kms_id.as_str(), &config).await?;
    let res = restore_secret_key(kms_id.as_str(), &config).await?;

    assert_eq!(secret, res);

    Ok(())
}
