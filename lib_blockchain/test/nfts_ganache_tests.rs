use crate::nfts_tests::MNEMONIC_TEST;
use chrono::Utc;
use ethers::utils::Ganache;
use lib_blockchain::blockchains::chain::CloneBoxNFTsRepository;
use lib_blockchain::blockchains::ganache::GanacheBlockChain;
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
use lib_blockchain::services::contract::deploy_evm_contract_locally;
use lib_blockchain::services::nfts::{NFTsManipulation, NFTsService, NTFState};
use lib_config::config::Config;
use lib_config::environment::{DEV_ENV, ENV_VAR_ENVIRONMENT};
use lib_config::infra::{
    build_local_stack_connection, create_key, create_secret_manager_with_values,
    cypher_with_secret_key,
};

use lib_blockchain::repositories::schema_blockchain::BlockchainSchema;
use lib_config::schema::Schema;

use spectral::{assert_that, result::ResultAssertions};
use std::env;
use testcontainers::*;
use url::Url;

#[tokio::test]
async fn create_contract_and_mint_nft_test_sync_ganache(
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env::set_var("RUST_LOG", "debug");
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

    //let creation = create_schema_blockchains(&dynamo_client).await;
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

    let ganache_params = vec!["-l 100000000".to_string()];
    let ganache = Ganache::new()
        .mnemonic(MNEMONIC_TEST)
        .args(ganache_params)
        .spawn();

    //Ethers
    // let aux_wallet: LocalWallet = ganache.keys()[0].clone().into();
    // let contract_owner_wallet = aux_wallet.with_chain_id( "1337".parse::<u64>()? );
    // let contract_owner_address  = format!("{:#?}", contract_owner_wallet.address());

    //Web3
    let contract_owner_secret: &str =
        "4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d"; // example fake secret key
    let key_id = config.env_vars().kms_key_id().unwrap();
    let contract_owner_secret_cyphered =
        cypher_with_secret_key(contract_owner_secret, key_id.as_str(), &config).await?;
    let contract_owner_address = "0x90F8bf6A479f320ead074411a4B0e7944Ea8c9C1".to_string(); //address based on the previous fake secret key

    //create blockchain ganache object and contract
    let block_chains_repo = BlockchainRepo::new(&config.clone());
    let contracts_repo = ContractRepo::new(&config.clone());

    //create contract and deploy to blockchain
    let url = ganache.endpoint();

    let contract_address =
        deploy_evm_contract_locally(url.as_str(), contract_owner_address.clone()).await?;
    //let contract_address = deploy_contract_ethers(url.as_str(), &contract_owner_wallet).await?;

    let confirmations = 0;
    let blochain_id = "ganache".to_string();

    let ganache_entity = Blockchain::new(
        blochain_id.to_owned(),
        Url::parse(url.as_str()).unwrap().clone(),
        "no-api-key".to_string(),
        confirmations,
        Url::parse("http://localhost/explorer").unwrap().clone(),
        "no-api-key-explorer".to_string(),
    );
    block_chains_repo.add(&ganache_entity).await?;

    let contact_id = 1;

    let contract_entity = Contract::new_c(
        contact_id,
        Utc::now(),
        blochain_id.to_owned(),
        Some(contract_address),
        Some(contract_owner_address),
        Some(contract_owner_secret_cyphered),
        Some("".to_string()),
        Some("no-details".to_string()),
        ContractStatus::Enabled,
    );
    contracts_repo.add(&contract_entity).await?;

    new_configuration.set_contract_id(contact_id);
    config.set_env_vars(&new_configuration);

    let blockchain = GanacheBlockChain::new(&config.clone(), &contracts_repo, &block_chains_repo)
        .await
        .unwrap();

    let nft_service = NFTsService::new(
        blockchain.clone_box(),
        repo_keys,
        tx_service.clone(),
        config.to_owned(),
    );

    let asset_price: u64 = 2000;

    let asset_id = uuid::Uuid::new_v4();
    let mint_op = nft_service
        .try_mint(
            &asset_id,
            &"user1".to_string(),
            &Some(asset_price),
            &"hash".to_string(),
            &"md5".to_string(),
            &0,
        )
        .await;
    assert_that!(&mint_op).is_ok();
    let tx_in_chain = mint_op.unwrap();

    let check_op = nft_service.get(&asset_id).await;
    assert_that!(&check_op).is_ok();
    let content = check_op.unwrap();

    assert_eq!(content.hash_file, "hash".to_string() );
    assert_eq!(content.price.unwrap(), asset_price);
    assert_eq!(content.state, NTFState::Active);

    let tx_tx = tx_service.get_by_id( &"hash".to_string() ).await;
    assert_that!(&tx_tx).is_ok();
    let final_tx = tx_tx.unwrap();
    assert_eq!(
        final_tx.mint_status(),
        MintingStatus::CompletedSuccessfully
    );
    let content1 = tx_in_chain.tx().clone().unwrap();
    let content2 = final_tx.tx().clone().unwrap();
    assert_eq!(content1, content2);

    let txs_op = tx_service.get_by_asset_id(&asset_id).await;
    assert_that!(&txs_op).is_ok();

    Ok(())
}
