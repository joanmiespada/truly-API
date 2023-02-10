use errors::AsyncOpError;
use tracing::log::debug;

pub mod errors;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[derive(Debug)]
pub struct SQSMessage {
    pub body: String,
    pub group: String,
}

async fn find(
    client: &aws_sdk_sqs::client::Client,
    config: &lib_config::config::Config,
) -> Result<String> {
    let queue_ops = client.list_queues().send().await;

    let queues = match queue_ops {
        Err(e) => return Err(AsyncOpError { 0: e.to_string() }.into()),
        Ok(rsp) => rsp,
    };

    let queue_urls = queues.queue_urls().unwrap_or_default();

    let url = config.env_vars().queue_mint_async().to_owned();
   
    let res = queue_urls.into_iter().filter( |x| **x == url).count();

    match res {
        1 => Ok(url.to_owned()),
        _ => Err( AsyncOpError{0:"no queue found".to_string()}.into() ) 
    }
}

pub async fn send(config: &lib_config::config::Config, message: &SQSMessage) -> Result<String> {
    let shared_config = config.aws_config();

    let client = aws_sdk_sqs::client::Client::new(shared_config);

    let queue_url = find(&client, config).await?;

    let rsp_op = client
        .send_message()
        .queue_url(queue_url)
        .message_body(&message.body)
        .message_group_id(&message.group)
        // If the queue is FIFO, you need to set .message_deduplication_id
        // or configure the queue for ContentBasedDeduplication.
        .send()
        .await;
    match rsp_op {
        Err(e) => Err(AsyncOpError { 0: e.to_string() }.into()),
        Ok(rsp) => {
            let result = format!("{:#?}", rsp);
            Ok(result)
        }
    }
}

pub async fn recieve(config: &lib_config::config::Config) -> Result<()>{//SQSMessage

    let shared_config = config.aws_config();
    let client = aws_sdk_sqs::client::Client::new(shared_config);
    let queue_url = find(&client, config).await?;

    let rcv_message_output = client.receive_message().queue_url(queue_url.clone()).send().await?;

    debug!("Messages from queue with url: {}", queue_url);

    for message in rcv_message_output.messages.unwrap_or_default() {
        debug!("Got the message: {:#?}", message);

    }

    Ok(())


}

pub async fn create(config: &lib_config::config::Config, name: String) -> Result<String> {
    let shared_config = config.aws_config();

    let client = aws_sdk_sqs::client::Client::new(shared_config);

    let res_op = client
        .create_queue()
        //.attributes(k, v)
        .queue_name(name)
        .tags("project", "truly")
        .send()
        .await;
    match res_op {
        Err(e) => Err(AsyncOpError { 0: e.to_string() }.into()),
        Ok(v) => {
            let res = v.queue_url().unwrap().to_owned();
            Ok(res)
        }
    }
}
pub async fn delete(config: &lib_config::config::Config, url: String) -> Result<()> {
    let shared_config = config.aws_config();

    let client = aws_sdk_sqs::client::Client::new(shared_config);

    let res_op = client
        .delete_queue()
        .queue_url( url )
        .send()
        .await;
    match res_op {
        Err(e) => Err(AsyncOpError { 0: e.to_string() }.into()),
        Ok(v) => {
            Ok(())
        }
    }
}