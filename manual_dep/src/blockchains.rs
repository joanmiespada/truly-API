use std::{fs::File, io::Read};

use aws_sdk_dynamodb::types::error::ResourceNotFoundException;
use lib_blockchain::{
    models::blockchain::Blockchain,
    repositories::{blockchain::{BlockchainRepo, BlockchainRepository} },
};

use lib_config::config::Config;

pub async fn manage_blockchains(
    blockchain_path: String,
    create: bool,
    delete: bool,
    _environment: String,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let er = ResourceNotFoundException::builder().build();

    if create {
        let mut file = File::open(blockchain_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let list_op = serde_json::from_str::<Vec<Blockchain>>(&contents);

        match list_op {
            Err(e) => {
                panic!("{}",e);
            }
            Ok(list) => {
                let block_chains_repo = BlockchainRepo::new(&config.clone());

                for item in list {
                    block_chains_repo.add(&item).await?;
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
async fn manage_blockchain_test() {
    use lib_config::{environment::DEV_ENV, infra::build_local_stack_connection};
    use spectral::{assert_that, result::ResultAssertions};
    use std::env;
    use testcontainers::{clients, images};
    use std::path::PathBuf;
    use lib_blockchain::repositories::schema_blockchain::create_schema_blockchains;

    env::set_var("RUST_LOG", "debug");
    env::set_var("ENVIRONMENT", "development");

    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();

    let mut local_stack = images::local_stack::LocalStack::default();
    local_stack.set_services("dynamodb");
    let node = docker.run(local_stack);
    let host_port = node.get_host_port_ipv4(4566);

    let shared_config = build_local_stack_connection(host_port).await;

    let client = aws_sdk_dynamodb::Client::new(&shared_config);
    let creation = create_schema_blockchains(&client).await;
    assert_that(&creation).is_ok();

    // set up config for truly app
    let mut config = Config::new();
    config.setup().await;
    config.set_aws_config(&shared_config); //rewrite configuration to use our current testcontainer instead
    //config.load_secrets().await;
    

    let filename = "manual_dep/res/blockchain_development.json";
    let current_dir = env::current_dir().unwrap();
    let mut file_path = PathBuf::from(current_dir);
    file_path.push(filename);
    let file_path = file_path.to_str().unwrap();

    let aux = manage_blockchains(
        file_path.to_string(),
        true,
        false,
        DEV_ENV.to_string(),
        &config,
    )
    .await;
    assert_that(&aux).is_ok();

    let block_chains_repo = BlockchainRepo::new(&config.clone());

    let obj = block_chains_repo.get_by_id(&"sui".to_string()).await;
    assert_that(&obj).is_ok();
}
