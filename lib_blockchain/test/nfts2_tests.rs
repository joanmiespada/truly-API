use crate::nfts_tests::MNEMONIC_TEST;
use chrono::Utc;
use ethers::utils::Ganache;
use lib_blockchain::blockchains::ganache::GanacheBlockChain;
use lib_blockchain::models::blockchain::Blockchain;
use lib_blockchain::models::contract::{Contract, ContractStatus};
use lib_blockchain::repositories::block_tx::BlockchainTxRepo;
use lib_blockchain::repositories::blockchain::{BlockchainRepo, BlockchainRepository};
use lib_blockchain::repositories::contract::{ContractRepo, ContractRepository};
use lib_blockchain::repositories::keypairs::KeyPairRepo;
use lib_blockchain::repositories::schema_block_tx::create_schema_transactions;
use lib_blockchain::repositories::schema_blockchain::create_schema_blockchains;
use lib_blockchain::repositories::schema_contract::create_schema_contracts;
use lib_blockchain::repositories::schema_keypairs::create_schema_keypairs;
use lib_blockchain::services::block_tx::{BlockchainTxManipulation, BlockchainTxService};
use lib_blockchain::services::contract::deploy_contract_locally;
use lib_blockchain::{
    services::nfts::{NFTsManipulation, NFTsService, NTFState},
};
use lib_config::config::Config;
use lib_config::infra::{
    build_local_stack_connection, create_key, create_secret_manager_keys,
    create_secret_manager_secret_key, store_secret_key,
};
use lib_licenses::models::asset::{MintingStatus, SourceType, VideoLicensingStatus};
use lib_licenses::repositories::assets::AssetRepo;
use lib_licenses::repositories::owners::OwnerRepo;
use lib_licenses::repositories::schema_asset::create_schema_assets_all;
use lib_licenses::repositories::schema_owners::create_schema_owners;
use lib_licenses::repositories::shorter::ShorterRepo;
use lib_licenses::services::assets::{AssetManipulation, AssetService, CreatableFildsAsset};
use lib_licenses::services::owners::OwnerService;

use spectral::{assert_that, result::ResultAssertions};
use std::{env, str::FromStr};
use testcontainers::*;
use url::Url;
use web3::types::{H160, H256};

#[tokio::test]
async fn create_contract_and_mint_nft_test_sync(
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env::set_var("RUST_LOG", "debug");
    env::set_var("ENVIRONMENT", "development");

    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();

    let mut local_stack = images::local_stack::LocalStack::default();
    local_stack.set_services("dynamodb,secretsmanager,kms");
    let node = docker.run(local_stack);
    let host_port = node.get_host_port_ipv4(4566);

    //create dynamodb tables against testcontainers.

    let shared_config = build_local_stack_connection(host_port).await;

    let dynamo_client = aws_sdk_dynamodb::Client::new(&shared_config);

    let creation1 = create_schema_blockchains(&dynamo_client).await;
    assert_that(&creation1).is_ok();

    let creation2 = create_schema_contracts(&dynamo_client).await;
    assert_that(&creation2).is_ok();

    let creation3 = create_schema_assets_all(&dynamo_client).await;
    assert_that(&creation3).is_ok();

    let creation4 = create_schema_owners(&dynamo_client).await;
    assert_that(&creation4).is_ok();

    let creation5 = create_schema_keypairs(&dynamo_client).await;
    assert_that(&creation5).is_ok();

    let creation6 = create_schema_transactions(&dynamo_client).await;
    assert_that(&creation6).is_ok();

    //create secrets and keys

    let keys_client = aws_sdk_kms::client::Client::new(&shared_config);
    let new_key_id = create_key(&keys_client).await?;
    env::set_var("KMS_KEY_ID", new_key_id.clone());

    let secrets_client = aws_sdk_secretsmanager::client::Client::new(&shared_config);
    let secrets_json = r#"
    {
        "HMAC_SECRET" : "localtest_hmac_1234RGsdfg#$%",
        "JWT_TOKEN_BASE": "localtest_jwt_sd543ERGds235$%^"
    }
    "#;
    create_secret_manager_keys(secrets_json, &secrets_client).await?;
    create_secret_manager_secret_key(&secrets_client).await?;

    // set up config for truly app
    let mut config = Config::new();
    config.setup().await;
    config.set_aws_config(&shared_config); //rewrite configuration to use our current testcontainer instead
    config.load_secrets().await;

    // bootstrap dependencies
    let repo_tx = BlockchainTxRepo::new(&config.clone());
    let tx_service = BlockchainTxService::new(repo_tx);

    let repo_ow = OwnerRepo::new(&config.clone());
    let owner_service = OwnerService::new(repo_ow);

    let repo_as = AssetRepo::new(&config.clone());
    let repo_sh = ShorterRepo::new(&config.clone());
    let asset_service = AssetService::new(repo_as, repo_sh);

    let repo_keys = KeyPairRepo::new(&config.clone());

    let mut new_configuration = config.env_vars().clone();

    //create fake test asset and user

    let asset_url: Url = Url::parse("http://www.file1.com/test1.mp4").unwrap();
    let asset_hash: String = "hash1234".to_string();
    let asset_license: String =
        String::from_str("license - open shared social networks - forbiden mass media").unwrap();

    let mut as0 = CreatableFildsAsset {
        url: asset_url.to_string(),
        hash: asset_hash,
        license: asset_license,
        longitude: None,
        latitude: None,
        father: None,
        source: SourceType::Others,
        source_details: None,
    };

    let user_id = String::from_str("user1234-1234-1234-1234").unwrap();

    let new_asset_op = asset_service.add(&mut as0, &user_id).await;

    assert_that!(&new_asset_op).is_ok();

    let mut as1 = asset_service
        .get_by_id(&new_asset_op.unwrap())
        .await
        .unwrap();

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
    let key_id = config.env_vars().kms_key_id();
    let contract_owner_secret_cyphered =
        store_secret_key(contract_owner_secret, key_id, &config).await?;
    let contract_owner_address = "0x90F8bf6A479f320ead074411a4B0e7944Ea8c9C1".to_string(); //address based on the previous fake secret key

    //create blockchain ganache object and contract
    let block_chains_repo = BlockchainRepo::new(&config.clone());
    let contracts_repo = ContractRepo::new(&config.clone());

    //create contract and deploy to blockchain
    let url = ganache.endpoint();

    let contract_address =
        deploy_contract_locally(url.as_str(), contract_owner_address.clone()).await?;
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
        Some(H160::from_str(contract_address.as_str()).unwrap()),
        Some(H160::from_str(contract_owner_address.as_str()).unwrap()),
        Some(contract_owner_secret_cyphered),
        Some("no-details".to_string()),
        ContractStatus::Enabled,
    );
    contracts_repo.add(&contract_entity).await?;

    // new_configuration.set_blockchain_url(url.clone());
    // new_configuration.set_contract_address(contract_address);
    // new_configuration.set_contract_owner_address(contract_owner_address.clone());
    new_configuration.set_contract_id(contact_id);
    config.set_env_vars(&new_configuration);

    let blockchain = GanacheBlockChain::new(&config.clone(), &contracts_repo, &block_chains_repo)
        .await
        .unwrap();

    let nft_service = NFTsService::new(
        blockchain,
        repo_keys,
        asset_service.clone(),
        owner_service.clone(),
        tx_service.clone(),
        config.to_owned(),
    );

    let asset_price: u64 = 2000;

    as1.set_video_licensing_status(VideoLicensingStatus::AlreadyLicensed);
    as1.set_counter(&Some(1));
    let update_op = asset_service.update_full(&as1).await;
    assert_that!(&update_op).is_ok();

    let mint_op = nft_service.try_mint(as1.id(), &user_id, &asset_price).await;
    assert_that!(&mint_op).is_ok();
    let tx_in_chain = mint_op.unwrap();

    let check_op = nft_service.get(as1.id()).await;
    assert_that!(&check_op).is_ok();
    let content = check_op.unwrap();

    assert_eq!(content.hash_file, as1.hash().as_deref().unwrap());
    assert_eq!(content.price, asset_price);
    assert_eq!(content.state, NTFState::Active);

    let tx_op = asset_service.get_by_id(as1.id()).await;

    assert_that!(&tx_op).is_ok();

    let content_minted = tx_op.unwrap();

    assert_eq!(
        *content_minted.mint_status(),
        MintingStatus::CompletedSuccessfully
    );
    assert_ne!(*content_minted.minted_tx(), None);

    let find = content_minted.minted_tx().clone().unwrap();
    let tx_find = H256::from_str(&find).unwrap();
    let tx_tx = tx_service.get_by_hash(&tx_find).await;
    assert_that!(&tx_tx).is_ok();
    let final_tx = tx_tx.unwrap();
    let content1 = tx_in_chain.tx().clone().unwrap();
    let content2 = final_tx.tx().clone().unwrap();
    assert_eq!(content1, content2);

    let txs_op = tx_service.get_by_asset_id(content_minted.id()).await;
    assert_that!(&txs_op).is_ok();
    let txs = txs_op.unwrap();
    assert_eq!(txs.len(), 1);

    Ok(())
}
