use aws_sdk_sns::types::Topic;
use tracing::log::info;

use crate::errors::AsyncOpError;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[derive(Debug)]
pub struct SNSMessage {
    pub body: String,
}

async fn _check_if_exist(
    client: &aws_sdk_sns::client::Client,
    _config: &lib_config::config::Config,
    topic_arn: String,
) -> Result<String> {
    let topics_ops = client.list_topics().send().await;

    let topics = match topics_ops {
        Err(e) => return Err(AsyncOpError { 0: e.to_string() }.into()),
        Ok(rsp) => rsp,
    };

    let topics_arns = topics.topics().unwrap_or_default();
    let my_topic = Topic::builder().topic_arn(topic_arn.to_owned()).build();

    let res = topics_arns.into_iter().filter(|x| **x == my_topic).count();

    match res {
        1 => Ok(topic_arn.to_owned()),
        _ => Err(AsyncOpError {
            0: "no queue found".to_string(),
        }
        .into()),
    }
}

#[tracing::instrument()]
pub async fn send(
    config: &lib_config::config::Config,
    message: &SNSMessage,
    topic_arn: String,
) -> Result<String> {
    let shared_config = config.aws_config();

    let client = aws_sdk_sns::client::Client::new(shared_config);

    info!("****************sending request {}", message.body);
    let rsp_op = client
        .publish()
        .topic_arn(topic_arn)
        .message(&message.body)
        .send()
        .await;
    match rsp_op {
        Err(e) => Err(AsyncOpError {
            0: e.into_service_error().meta().message().unwrap().to_string(),
        }
        .into()),
        Ok(rsp) => {
            let result = format!(
                "scheduled async miting for id {:?} successfully.",
                rsp.message_id().unwrap()
            );
            Ok(result)
        }
    }
}

// pub async fn subscribe_email(config: &lib_config::config::Config, queue_id: Url) -> Result<String>{

//     let shared_config = config.aws_config();
//     let client = aws_sdk_sns::client::Client::new(shared_config);
//     //let queue_url = find(&client, config).await?;
//     let queue_url = queue_id.to_string();

//     let rcv_message_output = client.receive_message().queue_url(queue_url.clone()).send().await?;

//     debug!("Messages from queue with url: {}", queue_url);

//     let message = rcv_message_output.messages.unwrap_or_default().first().unwrap().clone();
//     return Ok(message.body().unwrap().to_owned());

//     // for message in rcv_message_output.messages.unwrap_or_default() {
//     //     debug!("Got the message: {:#?}", message);
//     //     message.body()
//     // }

//     // Ok(())

// }

pub async fn create(config: &lib_config::config::Config, name: String) -> Result<String> {
    let shared_config = config.aws_config();

    let client = aws_sdk_sns::client::Client::new(shared_config);

    let res_op = client
        .create_topic()
        .name(name)
        .tags(
            aws_sdk_sns::types::Tag::builder()
                .key("project".to_owned())
                .value("truly".to_owned())
                .build(),
        )
        .send()
        .await;
    match res_op {
        Err(e) => Err(AsyncOpError { 0: e.to_string() }.into()),
        Ok(v) => {
            let aux = v.topic_arn().unwrap_or_default().to_string();
            Ok(aux)
        }
    }
}
pub async fn delete(config: &lib_config::config::Config, topic_arn: String) -> Result<()> {
    let shared_config = config.aws_config();

    let client = aws_sdk_sns::client::Client::new(shared_config);

    let res_op = client.delete_topic().topic_arn(topic_arn).send().await;
    match res_op {
        Err(e) => Err(AsyncOpError { 0: e.to_string() }.into()),
        Ok(_) => Ok(()),
    }
}
