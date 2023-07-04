// use aws_sdk_dynamodb::types::error::ResourceNotFoundException;
// use lib_async_ops::sns::create as create_topic;
// use lib_async_ops::sqs::create as create_queue;
// use lib_config::config::Config;

// pub async fn manage_async_jobs(
//     create: bool,
//     delete: bool,
//     _environment: String,
//     config: &Config,
// ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//     let er = ResourceNotFoundException::builder().build();
//     if create {
//         let name1 = "queue_minting_async".to_string();
//         let url1 = create_queue(&config, name1.to_owned()).await?;
//         println!("queue {} created at url: {}", name1, url1);

//         let name2 = "queue_minting_deathletter".to_string();
//         let url2 = create_queue(&config, name2.to_owned()).await?;
//         println!("queue {} created at url: {}", name2, url2);

//         let name3 = "topic_minting_async".to_string();
//         let arn = create_topic(&config, name3.to_owned()).await?;
//         println!("topic {} created at arn: {}", name2, arn);
//     } else if delete {
//         panic!("not implemented yet")
//     } else {
//         return Err(aws_sdk_dynamodb::Error::ResourceNotFoundException(er).into());
//     }

//     Ok(())
// }
