use std::{fs::File, io::Read};

use aws_sdk_dynamodb::types::error::ResourceNotFoundException;
use lib_blockchain::{models::blockchain::Blockchain, repositories::blockchain::{BlockchainRepo, BlockchainRepository}};
use lib_config::config::Config;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct BlockchainImporter {
    blockchains: Vec<Blockchain>,
}
pub async fn manage_blockchains(
    blockchain_path: String,
    create: bool,
    delete: bool,
    environment: String,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let er = ResourceNotFoundException::builder().build();

    let mut file = File::open(blockchain_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let list: BlockchainImporter = serde_json::from_str(&contents)?;

    let block_chains_repo = BlockchainRepo::new(&config.clone());

    for item in list.blockchains {
        block_chains_repo.add(&item).await?;
    }

    Ok(())
}
