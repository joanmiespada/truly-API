use std::{fs::File, io::Read};

use aws_sdk_dynamodb::types::error::ResourceNotFoundException;
use lib_blockchain::{
    models::contract::{Contract, ContractStatus},
    repositories::{contract::{ContractRepo, ContractRepository}, },
};
use lib_config::config::Config;

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
        let list = serde_json::from_str::<Vec<Contract>>(&contents)?;

        let contracts_repo = ContractRepo::new(&config.clone());

        for mut item in list {

            let id = item.id().clone();

            let check = contracts_repo.get_by_id(&id).await;

            match check {
                Err(_)=>{
                    item.set_status(&ContractStatus::Enabled);
                    contracts_repo.add(&item).await?;
                },
                Ok(_)=>{
                    contracts_repo.update(&item).await?;
                }
            }
        }
    } else if delete {
        panic!("not implemented yet")
    } else {
        return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
    }

    Ok(())
}

#[tokio::test]
async fn manage_contracts_test() {
    use lib_config::{environment::DEV_ENV, infra::build_local_stack_connection};
    use spectral::{assert_that, result::ResultAssertions};
    use std::env;
    use std::path::PathBuf;
    use testcontainers::{clients, images};
    use lib_blockchain::repositories::schema_contract::ContractSchema;
    use lib_config::schema::Schema;

    env::set_var("RUST_LOG", "debug");
    env::set_var("ENVIRONMENT", "development");

    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();

    let mut local_stack = images::local_stack::LocalStack::default();
    local_stack.set_services("dynamodb");
    let node = docker.run(local_stack);
    let host_port = node.get_host_port_ipv4(4566);

    let shared_config = build_local_stack_connection(host_port).await;
    //create dynamodb tables against testcontainers.

    let mut config = Config::new();
    config.setup().await;
    config.set_aws_config(&shared_config); //rewrite configuration to use our current testcontainer instead

    let creation = ContractSchema::create_schema(&config).await;
    assert_that(&creation).is_ok();


    let filename = "truly_cli/res/contract_development.json";
    let current_dir = env::current_dir().unwrap();
    let mut file_path = PathBuf::from(current_dir);
    file_path.push(filename);
    let file_path = file_path.to_str().unwrap();

    let aux = manage_contracts(
        file_path.to_string(),
        true,
        false,
        DEV_ENV.to_string(),
        &config,
    )
    .await;
    assert_that(&aux).is_ok();

    let contracts_repo = ContractRepo::new(&config.clone());

    let obj = contracts_repo.get_by_id(&1).await;
    assert_that(&obj).is_ok();
}
