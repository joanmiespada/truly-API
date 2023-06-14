use chrono::Utc;
use lib_blockchain::blockchains::chain::CloneBoxNFTsRepository;
use lib_blockchain::blockchains::sui::SuiBlockChain;
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
use lib_config::infra::{
    build_local_stack_connection, create_key, create_secret_manager_keys,
    create_secret_manager_secret_key, store_secret_key,
};
use lib_licenses::models::asset::{MintingStatus, SourceType, VideoLicensingStatus};
use lib_licenses::repositories::assets::AssetRepo;
use lib_licenses::repositories::owners::OwnerRepo;
use lib_licenses::repositories::schema_asset::AssetAllSchema;
use lib_licenses::repositories::schema_owners::OwnerSchema;
use lib_licenses::repositories::shorter::ShorterRepo;
use lib_licenses::services::assets::{AssetManipulation, AssetService, CreatableFildsAsset};
use lib_licenses::services::owners::OwnerService;

use lib_blockchain::repositories::schema_blockchain::BlockchainSchema;
use lib_config::schema::Schema;

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use spectral::{assert_that, result::ResultAssertions};
use std::{env, str::FromStr};
use sui_keys::keystore::{InMemKeystore, Keystore};
use testcontainers::*;
use url::Url;

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

    //tables

    //let creation1 = create_schema_blockchains(&dynamo_client).await;
    //assert_that(&creation1).is_ok();
    let creation = BlockchainSchema::create_schema(&config).await;
    assert_that(&creation).is_ok();

    let creation = ContractSchema::create_schema(&config).await;
    assert_that(&creation).is_ok();

    let creation = AssetAllSchema::create_schema(&config).await; 
    assert_that(&creation).is_ok();

    let creation = OwnerSchema::create_schema(&config).await;
    assert_that(&creation).is_ok();

    let creation = KeyPairSchema::create_schema(&config).await;
    assert_that(&creation).is_ok();

    let creation =  BlockTxSchema::create_schema(&config).await;
    assert_that(&creation).is_ok();

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
        hash_algorithm: "MD5".to_string(),
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
    let mut keystore = Keystore::from(InMemKeystore::new_insecure_for_tests(0));

    let contract_owner_address = SuiBlockChain::keystore_add_new_random_address(&mut keystore)?;

    let coin_address = airdrop(contract_owner_address.clone()).await?;

    let contract_owner_keystore: Vec<u8> = bincode::serialize(&keystore).unwrap();
    let contract_owner_secret_base64 =
        general_purpose::STANDARD_NO_PAD.encode(&contract_owner_keystore);
    let contract_owner_secret_cyphered =
        store_secret_key(&contract_owner_secret_base64, &new_key_id, &config).await?;
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
        asset_service.clone(),
        owner_service.clone(),
        tx_service.clone(),
        config.to_owned(),
    );

    as1.set_video_licensing_status(VideoLicensingStatus::AlreadyLicensed); // to be deleted
    as1.set_counter(&Some(1)); // to be deleted
    let update_op = asset_service.update_full(&as1).await;
    assert_that!(&update_op).is_ok();

    let mint_op = nft_service.try_mint(as1.id(), &user_id, &None).await;
    assert_that!(&mint_op).is_ok();
    let tx_in_chain = mint_op.unwrap();

    //let check_op = nft_service.get(as1.id()).await;
    //assert_that!(&check_op).is_ok();
    //let content = check_op.unwrap();

    //assert_eq!(content.hash_file, as1.hash().as_deref().unwrap());
    //assert_eq!(content.hash_algorithm, as1.hash_algorithm().as_deref().unwrap());
    //assert_eq!(content.state, NTFState::Active);

    let tx_op = asset_service.get_by_id(as1.id()).await;

    assert_that!(&tx_op).is_ok();

    let content_minted = tx_op.unwrap();

    assert_eq!(
        *content_minted.mint_status(),
        MintingStatus::CompletedSuccessfully
    );
    assert_ne!(*content_minted.minted_tx(), None);

    let find = content_minted.minted_tx().clone().unwrap();
    let tx_tx = tx_service.get_by_id(&find).await;
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
