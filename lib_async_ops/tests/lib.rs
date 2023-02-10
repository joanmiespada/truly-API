use lib_config::{infra::build_local_stack_connection, config::Config};
use spectral::{assert_that, result::ResultAssertions};
use std::{env, str::FromStr};
use testcontainers::*;

use lib_async_ops::{create, SQSMessage, send, recieve};



#[tokio::test]
async fn test_queues() ->  Result<(), Box<dyn std::error::Error>>{ 
    env::set_var("RUST_LOG", "debug");
    env::set_var("ENVIRONMENT", "development");

    env_logger::builder().is_test(true).init();


    let docker = clients::Cli::default();

    let mut local_stack = images::local_stack::LocalStack::default();
    local_stack.set_services("sqs");
    let node = docker.run(local_stack);
    let host_port = node.get_host_port_ipv4(4566);


    let shared_config = build_local_stack_connection(host_port).await;
    // set up config for truly app
    let mut config = Config::new();
    config.setup().await;
    config.set_aws_config(&shared_config); //rewrite configuration to use our current testcontainer instead
    
    let queue_url_op = create(&config, "test1".to_string()).await;
    assert_that( &queue_url_op ).is_ok();
    let queue_url = queue_url_op.unwrap();
    env::set_var("QUEUE_MINT_ASYNC", queue_url);
    
    config.setup().await;
    config.set_aws_config(&shared_config); //rewrite configuration to use our current testcontainer instead

    let content = SQSMessage{
       body: "body".to_string(),
       group: "Minters".to_string() 
    };

    let sent_op = send(&config, &content ).await;

    assert_that( &sent_op ).is_ok();
    let sent= sent_op.unwrap();
    println!("{}",sent);
    //let response = 
    let recv_op = recieve(&config).await;
    
    assert_that( &recv_op ).is_ok();
    let recv = recv_op.unwrap();

    //assert_eq!(content.body, response.body );
    //assert_eq!(content.group, response.group);

    Ok(())


}