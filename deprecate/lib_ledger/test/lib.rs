use std::env;

use chrono::Utc;
use lib_config::{
    config::Config,
    environment::{DEV_ENV, ENV_VAR_ENVIRONMENT, STAGE_ENV},
    infra::build_local_stack_connection,
    result::ResultE,
    schema::Schema,
};
use lib_ledger::{
    models::AssetLedged,
    repository::{schema_ledger::LedgerSchema, LedgerRepo},
    service::{LedgerManipulation, LedgerService},
};
use spectral::{assert_that, result::ResultAssertions};
use testcontainers::*;
use uuid::Uuid;

use rand::distributions::Alphanumeric;
use rand::Rng;

#[tokio::test]
//TODO: qldb client can't aim to localstack endpoint.
async fn asset_test_local() -> ResultE<()> {
    //dotenv::dotenv().ok();
    dotenv::from_filename(".env-stage").ok();

    env::set_var("RUST_LOG", "debug");
    env::set_var(ENV_VAR_ENVIRONMENT, DEV_ENV);
    env::set_var("AWS_REGION", "eu-central-1");
    env::set_var("AWS_PROFILE", "localstack");

    let local_stack_api_key = env::var("LOCALSTACK_API_KEY").unwrap();

    let _ = env_logger::builder().is_test(true).try_init();

    let docker = clients::Cli::default();

    let local_stack = LocalStackProBuilder::defaults(local_stack_api_key).build();

    let node = docker.run(local_stack);
    let host_port = node.get_host_port_ipv4(4566);

    let shared_config = build_local_stack_connection(host_port).await;
    env::set_var("AWS_ENDPOINT", shared_config.endpoint_url().unwrap());

    let mut config = Config::new();
    config.setup().await;
    config.set_aws_config(&shared_config); //rewrite configuration to use our current testcontainer instead

    let creation = LedgerSchema::create_schema(&config).await;
    assert_that(&creation).is_ok();

    let ledger_repo = LedgerRepo::new(&config);
    let ledger_service = LedgerService::new(ledger_repo);

    let new_asset = AssetLedged {
        asset_id: Uuid::new_v4(),
        asset_hash: "hash".to_string(),
        asset_hash_algorithm: "hash_algorithm".to_string(),
        asset_creation_time: Utc::now(),
    };

    let ledge_data = ledger_service.add(&new_asset).await;
    assert_that(&ledge_data).is_ok();
    let ledge_data = ledger_service.add(&new_asset).await;
    assert_that(&ledge_data).is_err();

    let asset2 = ledger_service
        .get_by_hash(&new_asset.asset_hash.clone())
        .await;
    assert_that(&asset2).is_ok();
    let asset22 = ledger_service
        .get_by_asset_id(&new_asset.asset_id.clone())
        .await;
    assert_that(&asset22).is_ok();

    Ok(())
}

#[tokio::test]
async fn asset_prod_test() -> ResultE<()> {
    env::set_var("RUST_LOG", "debug");
    env::set_var(ENV_VAR_ENVIRONMENT, STAGE_ENV);
    env::set_var("AWS_REGION", "eu-west-1");
    env::set_var("AWS_PROFILE", "truly");

    let mut config = Config::new();
    config.setup().await;

    //remove this comments if you want to test the creation too. But in AWS, create a ledger requires time.
    //let creation = LedgerSchema::create_schema(&config).await;
    //assert_that(&creation).is_ok();

    let ledger_repo = LedgerRepo::new(&config);
    let ledger_service = LedgerService::new(ledger_repo);

    let random_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32) // You can change the length of the string here.
        .map(char::from)
        .collect();

    let new_asset = AssetLedged {
        asset_id: Uuid::new_v4(),
        asset_hash: random_string,
        asset_hash_algorithm: "hash_algorithm".to_string(),
        asset_creation_time: Utc::now(),
    };

    println!("id to be inserted {} ", new_asset.asset_id);

    let ledge_data = ledger_service.add(&new_asset).await;
    assert_that(&ledge_data).is_ok();
    let ledge_data = ledger_service.add(&new_asset).await;
    assert_that(&ledge_data).is_err();

    let asset2 = ledger_service
        .get_by_hash(&new_asset.asset_hash.clone())
        .await;
    assert_that(&asset2).is_ok();
    let asset22 = ledger_service
        .get_by_asset_id(&new_asset.asset_id)
        .await;
    assert_that(&asset22).is_ok();

    Ok(())
}
