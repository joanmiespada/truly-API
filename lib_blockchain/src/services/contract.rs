use std::{str::FromStr, time::Duration};

use web3::{
    contract::{Contract, Options},
    types::{H160, U256},
};

/// run it after local ganache bootstrapped.

pub async fn deploy_contract_locally(
    url: &str,
    contract_owner_address: String,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let http = web3::transports::Http::new(url)?;
    let web3 = web3::Web3::new(http);

    let gas_price = web3.eth().gas_price().await.unwrap();
    //let chain_id = web3.eth().chain_id().await.unwrap().as_u64();

    let bytecode = include_str!("../../res/evm/LightNFT.bin").trim_end();
    let abi = include_bytes!("../../res/evm/LightNFT.abi");

    let contract_deploy = Contract::deploy(web3.eth(), abi)
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
        .await?;

    let contract_address = format!("{:?}", contract_deploy.address());

    return Ok(contract_address);
}
