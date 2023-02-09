use crate::build_local_stack_connection;
use crate::nfts_tests::{store_secret_key, deploy_contract_web3, MNEMONIC_TEST,create_secrets,create_key};
use ethers::utils::Ganache;
use lib_config::Config;
use lib_licenses::repositories::assets::AssetRepo;
use lib_licenses::repositories::keypairs::KeyPairRepo;
use lib_licenses::repositories::owners::OwnerRepo;
use lib_licenses::repositories::schema_asset::create_schema_assets;
use lib_licenses::repositories::schema_keypairs::create_schema_keypairs;
use lib_licenses::repositories::schema_owners::create_schema_owners;
use lib_licenses::services::assets::{AssetManipulation, AssetService, CreatableFildsAsset};
use lib_licenses::services::owners::OwnerService;
use lib_licenses::{
    repositories::ganache::GanacheRepo,
    services::nfts::{NFTsManipulation, NFTsService, NTFState},
};

use spectral::{assert_that, result::ResultAssertions};
use std::{env, str::FromStr};
use testcontainers::*;
use url::Url;



#[tokio::test]
async fn create_contract_and_mint_nft_test_sync() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "debug");
    env::set_var("ENVIRONMENT", "development");

    env_logger::builder().is_test(true).init();


    let docker = clients::Cli::default();

    //let node = docker.run(images::dynamodb_local::DynamoDb::default());
    //let host_port = node.get_host_port_ipv4(8000);

    let mut local_stack = images::local_stack::LocalStack::default();
    local_stack.set_services("dynamodb,secretsmanager,kms");
    let node = docker.run(local_stack);
    let host_port = node.get_host_port_ipv4(4566);

    //create dynamodb tables against testcontainers.

    let shared_config = build_local_stack_connection(host_port).await;

    let dynamo_client = aws_sdk_dynamodb::Client::new(&shared_config);

    let creation1 = create_schema_assets(&dynamo_client).await;
    assert_that(&creation1).is_ok();

    let creation2 = create_schema_owners(&dynamo_client).await;
    assert_that(&creation2).is_ok();

    let creation3 = create_schema_keypairs(&dynamo_client).await;
    assert_that(&creation3).is_ok();

    //create secrets and keys
    let secrets_client =aws_sdk_secretsmanager::Client::new(&shared_config);
    create_secrets(&secrets_client).await?;
    let keys_client = aws_sdk_kms::Client::new(&shared_config);
    let new_key_id = create_key(&keys_client).await?;
    env::set_var("KMS_KEY_ID", new_key_id);

    // set up config for truly app
    let mut config = Config::new();
    config.setup().await;
    config.set_aws_config(&shared_config); //rewrite configuration to use our current testcontainer instead
    config.load_secrets().await;

    

    // bootstrap dependencies

    let repo_ow = OwnerRepo::new(&config.clone());
    let owner_service = OwnerService::new(repo_ow);

    let repo_as = AssetRepo::new(&config.clone());
    let asset_service = AssetService::new(repo_as);

    let repo_keys = KeyPairRepo::new(&config.clone());

    //restore connection dependencies to work with localstack
    config = Config::new();
    config.setup_with_secrets().await;


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
    };

    let user_id = String::from_str("user1234-1234-1234-1234").unwrap();

    let new_asset_op = asset_service.add(&mut as0, &user_id).await;

    assert_that!(&new_asset_op).is_ok();

    let as1 = asset_service
        .get_by_id(&new_asset_op.unwrap())
        .await
        .unwrap();

    //Create contract owner account

    let ganache = Ganache::new().mnemonic(MNEMONIC_TEST).spawn();

    //Ethers
    // let aux_wallet: LocalWallet = ganache.keys()[0].clone().into();
    // let contract_owner_wallet = aux_wallet.with_chain_id( "1337".parse::<u64>()? );
    // let contract_owner_address  = format!("{:#?}", contract_owner_wallet.address());

    //Web3
    let secret: &str = "4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d"; // secret key
    let key_id = config.env_vars().kms_key_id();
    store_secret_key(secret, key_id, &config).await?;
    let contract_owner_address = "0x90F8bf6A479f320ead074411a4B0e7944Ea8c9C1".to_string();
    //let contract_owner_private_key = SecretKey::from_str(secret).unwrap();
    //let contract_owner_private_key = SecretKeyRef::new(&contract_owner_private_key);
    //let p :SecretKey = SecretKey::from_str(private_key_0).unwrap();
    //let p_aux: LocalWallet = LocalWallet::from_str(private_key_0 ).unwrap(); //  p.clone().into();

    // let wallet: LocalWallet = "4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d"
    //     .parse::<LocalWallet>()?
    //     .with_chain_id(....);

    //let contract_owner_private_key = format!("{:?}", ganache.keys()[0] );

    //create contract and deploy to blockchain
    let url = ganache.endpoint();

    let contract_address =
        deploy_contract_web3(url.as_str(), contract_owner_address.clone()).await?;
    //let contract_address = deploy_contract_ethers(url.as_str(), &contract_owner_wallet).await?;

    new_configuration.set_blockchain_url(url.clone());
    new_configuration.set_contract_address(contract_address);
    new_configuration.set_contract_owner_address(contract_owner_address.clone());
    config.set_env_vars(&new_configuration);

    let blockchain = GanacheRepo::new(&config.clone()).unwrap();


    let nft_service = NFTsService::new(
        blockchain,
        repo_keys,
        asset_service.clone(),
        owner_service.clone(),
    );

    let price: u64 = 2000;

    let mint_op = nft_service.add(as1.id(), &user_id, &price).await;

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


