use crate::models::subscription::Subscription;
use crate::template::intent::get_intent_message;
use crate::template::new_content_found::get_similar_content_found_message;
use lettre::message::{header, MultiPart, SinglePart, Mailbox};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport, Address};
use lib_config::config::Config;
use lib_config::result::ResultE;
use lib_licenses::models::asset::Asset;
use lib_users::models::user::User;
use url::Url;
use uuid::Uuid;

pub const SMTP_TEST_SERVER: &str = "test.test.test";

pub struct SenderEmailsRepo {
    smtp_host: String,
    smtp_user: String,
    smtp_passw: String,
}

impl SenderEmailsRepo {
    pub fn new(conf: &Config) -> Self {
        SenderEmailsRepo {
            smtp_host: conf.env_vars().smtp_host().unwrap().to_owned(),
            smtp_user: conf.env_vars().smtp_user().unwrap().to_owned(),
            smtp_passw: conf.env_vars().smtp_passw().unwrap().to_owned(),
        }
    }

    async fn send(
        &self,
        email: String,
        subject: String,
        body_flat_text: String,
        body_html: String,
    ) -> ResultE<()> {

        let from = "Joan <joan@mail1.truly.video>".parse()?;

        //let to_op = format!("{} <{}>", email, email);

        let to = match email.parse::<Address>() {
            Ok(address) => Mailbox::from(address),
            Err(e) => {
                log::error!("Could not parse email: {:?}", e);
                return Err(Box::new(e));
            }
        };



        let message_op = Message::builder()
            // Addresses can be specified by the tuple (email, alias)
            .to(to)
            .from(from)
            .subject(subject)
            .multipart(
                MultiPart::related()
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_HTML)
                            .body(body_html),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_PLAIN)
                            .body(body_flat_text),
                    ),
            );
        if let Err(e) = message_op {
                log::error!("Could not create email: {:?}", e);
                return Err(Box::new(e));
        }

        let message = message_op.unwrap();

        let creds = Credentials::new(
            self.smtp_user.clone(),
            self.smtp_passw.clone()
        );

        let smtp_host = self.smtp_host.clone(); 

        if smtp_host == SMTP_TEST_SERVER {
            log::info!("Email sent to: {}", email);
            return Ok(());
        }

        let mailer = SmtpTransport::relay(smtp_host.as_str())?
            .credentials(creds)
            .build();

        // Send the email
        match mailer.send(&message) {
            Ok(_) => {
                //TODO: store email has been sent at DynamoDB
            }
            Err(e) => {
                //panic!("Could not send email: {e:?}")
                log::error!("Could not send email: {:?}", e)
            }
        }

        Ok(())
    }

    //#[instrument]
    pub async fn send_intent(
        &self,
        user: User, 
        asset: Asset,
        subscription: Subscription,
    ) -> ResultE<()> {

        log::info!("Sending intent email to: {}", user);

        let email = user.email().clone().unwrap();
        let url = asset.url().clone().unwrap();

        let (subject, body_flat_text, body_html) =
            get_intent_message( email.clone(), url, subscription.id);

        self.send( email, subject, body_flat_text, body_html).await
    }

    pub async fn send_new_similar_content_found(
        &self,
        email: String,
        asset_subscribed: Url,
        asset_similars: Vec<(Url, Uuid)>,
    ) -> ResultE<()> {
        let (subject, body_flat_text, body_html) = get_similar_content_found_message(
            email.clone(),
            asset_subscribed,
            asset_similars,
        );

        self.send( email, subject, body_flat_text, body_html).await
    }
}
