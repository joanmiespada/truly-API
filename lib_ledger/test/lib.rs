use std::env;

use lib_config::{result::ResultE, environment::{ENV_VAR_ENVIRONMENT, DEV_ENV}, infra::build_local_stack_connection, config::Config, schema::Schema};
use lib_ledger::{repository::{schema_ledger::LedgerSchema, LedgerRepo}, service::{LedgerService, LedgerManipulation}};
use lib_licenses::models::asset::Asset;
use spectral::{assert_that, result::ResultAssertions};
use testcontainers::*;

#[tokio::test]
async fn asset_test() -> ResultE<()> {

    //dotenv::dotenv().ok();
    dotenv::from_filename(".env-stage").ok();

    env::set_var("RUST_LOG", "debug");
    env::set_var(ENV_VAR_ENVIRONMENT, DEV_ENV);
    env::set_var("AWS_REGION", "eu-central-1");

    let local_stack_api_key = env::var("LOCALSTACK_API_KEY").unwrap();

    let _ = env_logger::builder().is_test(true).try_init();

    let docker = clients::Cli::default();

    let local_stack =  LocalStackProBuilder::defaults(local_stack_api_key).build(); 

    let node = docker.run(local_stack);
    let host_port = node.get_host_port_ipv4(4566);

    let shared_config = build_local_stack_connection(host_port).await;

    let mut config = Config::new();
    config.setup().await;
    config.set_aws_config(&shared_config); //rewrite configuration to use our current testcontainer instead

    let creation = LedgerSchema::create_schema(&config).await;
    assert_that(&creation).is_ok();
    let creation = LedgerSchema::create_table(&config).await;
    assert_that(&creation).is_ok();

    let ledger_repo = LedgerRepo::new(&config);
    let ledger_service = LedgerService::new(ledger_repo);

    let mut new_asset = Asset::new();

    let ledge_data = ledger_service.add(&new_asset).await?;

    let asset2 = ledger_service.get_by_id(&ledge_data.tx).await?;

    Ok(())

    //assert_eq!(new_asset.status(), asset2.status());
}