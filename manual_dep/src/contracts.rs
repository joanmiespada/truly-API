use std::{fs::File, io::Read};

use aws_sdk_dynamodb::types::error::ResourceNotFoundException;
use lib_blockchain::{
    models::contract::{Contract, ContractStatus},
    repositories::contract::{ContractRepo, ContractRepository},
};
use lib_config::config::Config;
use serde::Deserialize;
#[derive(Deserialize, Debug)]
struct ContractImporter {
    contracts: Vec<Contract>,
}
pub async fn manage_contracts(
    contract_path: String,
    create: bool,
    delete: bool,
    _environment: String,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let er = ResourceNotFoundException::builder().build();
    if create {
        let mut file = File::open(contract_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let list: ContractImporter = serde_json::from_str(&contents)?;

        let contracts_repo = ContractRepo::new(&config.clone());

        for mut item in list.contracts {
            item.set_status(&ContractStatus::Enabled);
            contracts_repo.add(&item).await?;
        }
    } else if delete {
        panic!("not implemented yet")
    } else {
        return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
    }

    Ok(())
}
