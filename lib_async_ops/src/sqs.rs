use std::str::FromStr;

use tracing::log::debug;
use url::Url;

use crate::errors::AsyncOpError;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[derive(Debug)]
pub struct SQSMessage {
    pub id: String,
    pub body: String,
}

async fn _check_if_exist(
    client: &aws_sdk_sqs::client::Client,
    _config: &lib_config::config::Config,
    queue_id: Url,
) -> Result<String> {
    let queue_ops = client.list_queues().send().await;

    let queues = match queue_ops {
        Err(e) => return Err(AsyncOpError { 0: e.to_string() }.into()),
        Ok(rsp) => rsp,
    };

    let queue_urls = queues.queue_urls().unwrap_or_default();

    let url = queue_id.to_string();
    // let url = config.env_vars().queue_mint_async().to_owned();

    let res = queue_urls.into_iter().filter(|x| **x == url).count();

    match res {
        1 => Ok(url.to_owned()),
        _ => Err(AsyncOpError {
            0: "no queue found".to_string(),
        }
        .into()),
    }
}

pub async fn send(
    config: &lib_config::config::Config,
    message: &SQSMessage,
    queue_id: Url,
) -> Result<String> {
    let shared_config = config.aws_config();

    let client = aws_sdk_sqs::client::Client::new(shared_config);

    //let queue_url = find(&client, config).await?;
    let queue_url = queue_id.to_string();

    let rsp_op = client
        .send_message()
        .queue_url(queue_url)
        .message_body(&message.body)
        //.message_group_id(&message.group)
        .message_deduplication_id(&message.id)
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

pub async fn recieve(config: &lib_config::config::Config, queue_id: Url) -> Result<String> {
    //SQSMessage

    let shared_config = config.aws_config();
    let client = aws_sdk_sqs::client::Client::new(shared_config);
    //let queue_url = find(&client, config).await?;
    let queue_url = queue_id.to_string();

    let rcv_message_output = client
        .receive_message()
        .queue_url(queue_url.clone())
        .send()
        .await?;

    debug!("Messages from queue with url: {}", queue_url);

    let message = rcv_message_output
        .messages
        .unwrap_or_default()
        .first()
        .unwrap()
        .clone();
    return Ok(message.body().unwrap().to_owned());

    // for message in rcv_message_output.messages.unwrap_or_default() {
    //     debug!("Got the message: {:#?}", message);
    //     message.body()
    // }

    // Ok(())
}

pub async fn create(config: &lib_config::config::Config, name: String) -> Result<Url> {
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
            let aux = v.queue_url().unwrap().to_owned();
            let res = Url::from_str(&aux).unwrap();
            Ok(res)
        }
    }
}
pub async fn delete(config: &lib_config::config::Config, queue_id: Url) -> Result<()> {
    let shared_config = config.aws_config();

    let client = aws_sdk_sqs::client::Client::new(shared_config);

    let res_op = client.delete_queue().queue_url(queue_id).send().await;
    match res_op {
        Err(e) => Err(AsyncOpError { 0: e.to_string() }.into()),
        Ok(_) => Ok(()),
    }
}
