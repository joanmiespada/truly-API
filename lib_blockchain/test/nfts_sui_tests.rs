use chrono::Utc;
use lib_blockchain::blockchains::chain::CloneBoxNFTsRepository;
use lib_blockchain::blockchains::sui::SuiBlockChain;
use lib_blockchain::models::block_tx::MintingStatus;
use lib_blockchain::models::blockchain::Blockchain;
use lib_blockchain::models::contract::{Contract, ContractStatus};
use lib_blockchain::repositories::block_tx::BlockchainTxRepo;
use lib_blockchain::repositories::blockchain::{BlockchainRepo, BlockchainRepository};
use lib_blockchain::repositories::contract::{ContractRepo, ContractRepository};
use lib_blockchain::repositories::keypairs::KeyPairRepo;
use lib_blockchain::repositories::schema_block_tx::BlockTxSchema;
use lib_blockchain::repositories::schema_contract::ContractSchema;
use lib_blockchain::repositories::schema_keypairs::KeyPairSchema;
use lib_blockchain::services::block_tx::{BlockchainTxManipulation, BlockchainTxService};
use lib_blockchain::services::nfts::{NFTsManipulation, NFTsService};
use lib_config::config::Config;
use lib_config::environment::{DEV_ENV, ENV_VAR_ENVIRONMENT};
use lib_config::infra::{
    build_local_stack_connection, create_key, create_secret_manager_with_values,
    cypher_with_secret_key,
};

use lib_blockchain::repositories::schema_blockchain::BlockchainSchema;
use lib_config::schema::Schema;

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use spectral::{assert_that, result::ResultAssertions};
use std::env;
use sui_keys::keystore::{InMemKeystore, Keystore};
use testcontainers::*;
use url::Url;

const ENV_VAR_AWS_REGION: &str = "AWS_REGION";
const TEST_AWS_REGION: &str = "eu-central-1";

#[tokio::test]
async fn create_contract_and_mint_nft_test_sync_sui(
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env::set_var("RUST_LOG", "debug");
    env::set_var(ENV_VAR_AWS_REGION, TEST_AWS_REGION);
    env::set_var(ENV_VAR_ENVIRONMENT, DEV_ENV);

    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();

    let mut local_stack = images::local_stack::LocalStack::default();
    local_stack.set_services("dynamodb,secretsmanager,kms");
    let node = docker.run(local_stack);
    let host_port = node.get_host_port_ipv4(4566);

    //create dynamodb tables against testcontainers.

    let shared_config = build_local_stack_connection(host_port).await;
    // set up config for truly app
    let mut config = Config::new();
    config.setup().await;
    config.set_aws_config(&shared_config); //rewrite configuration to use our current testcontainer instead
                                           //create secrets and keys

    //let keys_client = aws_sdk_kms::client::Client::new(&shared_config);
    let new_key_id = create_key(&config).await?;
    env::set_var("KMS_KEY_ID", new_key_id.clone());

    //let secrets_client = aws_sdk_secretsmanager::client::Client::new(&shared_config);
    let secrets_json = r#"
    {
        "HMAC_SECRET" : "localtest_hmac_1234RGsdfg#$%",
        "JWT_TOKEN_BASE": "localtest_jwt_sd543ERGds235$%^"
    }
    "#;
    create_secret_manager_with_values(secrets_json, &config).await?;

    config.load_secrets().await;

    //tables

    let creation = BlockchainSchema::create_schema(&config).await;
    assert_that(&creation).is_ok();

    let creation = ContractSchema::create_schema(&config).await;
    assert_that(&creation).is_ok();

    let creation = KeyPairSchema::create_schema(&config).await;
    assert_that(&creation).is_ok();

    let creation = BlockTxSchema::create_schema(&config).await;
    assert_that(&creation).is_ok();

    // bootstrap dependencies
    let repo_tx = BlockchainTxRepo::new(&config.clone());
    let tx_service = BlockchainTxService::new(repo_tx);

    let repo_keys = KeyPairRepo::new(&config.clone());

    let mut new_configuration = config.env_vars().clone();

    //Create contract owner account
    let mut keystore = Keystore::from(InMemKeystore::new_insecure_for_tests(0));

    let contract_owner_address = SuiBlockChain::keystore_add_new_random_address(&mut keystore)?;

    let coin_address = airdrop(contract_owner_address.clone()).await?;

    let contract_owner_keystore: Vec<u8> = bincode::serialize(&keystore).unwrap();
    let contract_owner_secret_base64 =
        general_purpose::STANDARD_NO_PAD.encode(&contract_owner_keystore);
    let contract_owner_secret_cyphered =
        cypher_with_secret_key(&contract_owner_secret_base64, &new_key_id, &config).await?;
    //create blockchain object and contract
    let block_chains_repo = BlockchainRepo::new(&config.clone());
    let contracts_repo = ContractRepo::new(&config.clone());

    // we need to bootstrap sui blockchain and sui faucet manually
    //create contract and deploy to blockchain
    let url = "http://127.0.0.1:9000".to_string();

    // it needs to be taken from the manual contrat deployment
    let contract_address =
        "0x5dd2881df4e9f44495a4d44dc6d24ec486c7f2c13b0701b68462b34f79530f57".to_string();

    let confirmations = 0;
    let blochain_id = "sui".to_string();

    let blockchain_entity = Blockchain::new(
        blochain_id.to_owned(),
        Url::parse(url.as_str()).unwrap().clone(),
        "no-api-key".to_string(),
        confirmations,
        Url::parse("https://suiexplorer.com/?network=local")
            .unwrap()
            .clone(),
        "no-api-key-explorer".to_string(),
    );
    block_chains_repo.add(&blockchain_entity).await?;

    let contact_id = 1;

    let contract_entity = Contract::new_c(
        contact_id,
        Utc::now(),
        blochain_id.to_owned(),
        Some(contract_address),
        Some(contract_owner_address),
        Some(contract_owner_secret_cyphered),
        Some(coin_address),
        Some("sui blockchain".to_string()),
        ContractStatus::Enabled,
    );
    contracts_repo.add(&contract_entity).await?;

    new_configuration.set_contract_id(contact_id);
    config.set_env_vars(&new_configuration);

    let blockchain = SuiBlockChain::new(&config.clone(), &contracts_repo, &block_chains_repo)
        .await
        .unwrap();

    let nft_service = NFTsService::new(
        blockchain.clone_box(),
        repo_keys,
        tx_service.clone(),
        config.to_owned(),
    );
    let asset_id = uuid::Uuid::new_v4();
    let mint_op = nft_service
        .try_mint(
            &asset_id,
            &"user1".to_string(),
            &None,
            &"hash".to_string(),
            &"md5".to_string(),
            &0,
        )
        .await;
    assert_that!(&mint_op).is_ok();
    let tx_in_chain = mint_op.unwrap();

    assert_eq!(
        tx_in_chain.mint_status(),
        MintingStatus::CompletedSuccessfully
    );

    let tx_tx = tx_service.get_by_id(&"hash".to_string()).await;
    assert_that!(&tx_tx).is_ok();
    let final_tx = tx_tx.unwrap();
    let content1 = tx_in_chain.tx().clone().unwrap();
    let content2 = final_tx.tx().clone().unwrap();
    assert_eq!(content1, content2);

    let txs_op = tx_service.get_by_asset_id(&asset_id).await;
    assert_that!(&txs_op).is_ok();

    Ok(())
}

async fn airdrop(
    contract_owner_address: String,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    #[derive(Serialize, Debug)]
    struct FixedAmountRequest {
        #[serde(rename = "FixedAmountRequest")]
        pub fixed_amount_request: FixedAmountRequestType,
    }
    #[derive(Serialize, Debug)]
    struct FixedAmountRequestType {
        pub recipient: String,
    }

    #[derive(Deserialize, Debug, Clone)]
    struct Item {
        #[serde(rename = "amount")]
        pub _amount: u128,
        pub id: String,
        #[serde(rename = "transferTxDigest")]
        pub _transfer_tx_digest: String,
    }
    #[derive(Deserialize, Debug, Clone)]
    struct ResultFixedAmountRequest {
        #[serde(rename = "transferredGasObjects")]
        pub transferred_gas_objects: Vec<Item>,
    }

    //airdrop my address
    let aux = FixedAmountRequest {
        fixed_amount_request: FixedAmountRequestType {
            recipient: contract_owner_address.to_string(),
        },
    };
    //let serialized = serde_json::to_string(&aux).unwrap();

    let client = reqwest::Client::new();
    let aux_aux = serde_json::to_string(&aux).unwrap();
    //println!("{:#?}", aux_aux);
    let req = client
        .post("http://127.0.0.1:9123/gas")
        .header("Content-Type", "application/json")
        .body(aux_aux);

    //println!("{:?}",req);
    let resp_op = req.send().await;
    if let Err(e) = resp_op {
        panic!("error calling faucet {}", e);
    }
    let resp = resp_op.ok().unwrap();

    //println!("{:#?}", resp);

    let aux = resp.json::<ResultFixedAmountRequest>().await?;

    //println!("{:#?}", aux.clone());

    let coin_address = aux.transferred_gas_objects[0].id.clone();

    Ok(coin_address)
}
