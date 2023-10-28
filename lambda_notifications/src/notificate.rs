use std::collections::HashMap;
use lib_config::config::Config;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use url::Url;
use uuid::Uuid;

pub type FoundSimilars= HashMap<std::string::String, HashMap<Url, HashMap<Url, Uuid>>>;


//#[instrument]
pub async fn send_notifications(
    conf: &Config,
    messages: FoundSimilars
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {


    for item in messages{
        let email = item.0;
        let asset_subscribed = item.1.keys().next().unwrap().to_owned();
        let asset_similars = item.1.values().next().unwrap().keys().map(|asset| asset.to_owned()).collect::<Vec<Url>>();


        let to = format!("{} <{}>", email, email);

        let asset_similars_text = asset_similars.iter().map(|asset| format!(" - {}\n", asset)).collect::<String>();

        let body = format!(r#"
        Hi {email},
        
        We have found similar videos to your video: {asset_subscribed}
        Please review: 
        
        {asset_similars}

        If you've got any dubts, please, don't hesitate to contact us by our Discord channel: https://disboard.org/server/1164515811390664735 
        Your feedback is really appreciated.

        Joan from truly.video
        "#, email=email, asset_subscribed = asset_subscribed, asset_similars=asset_similars_text );


        let email = Message::builder()
            // Addresses can be specified by the tuple (email, alias)
            .to(to.parse()? )
            .from("Joan <joan@mail1.truly.video>".parse()? )
            .subject("Truly.video has found similar videos to your video")
            .body(body)
            .unwrap();


        let creds = Credentials::new(
            conf.env_vars().smtp_user().unwrap(), // "smtp_username".to_owned(),
            conf.env_vars().smtp_passw().unwrap() //"smtp_password".to_owned());
        );

        let smtp_host = conf.env_vars().smtp_host().unwrap().to_owned();
        // Open a remote connection to gmail
        let mailer = SmtpTransport::relay(smtp_host.as_str())
            .unwrap()
            .credentials(creds)
            .build();

        // Send the email
        match mailer.send(&email) {
            Ok(_) => {
                //TODO: store email has been sent at DynamoDB
            },
            Err(e) => {
                //panic!("Could not send email: {e:?}")
                log::error!("Could not send email: {e:?}")
            },
        }

    }

   
    Ok(())
}
