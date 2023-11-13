use crate::Notificator;
use lib_engage::repositories::sender::SenderEmailsRepo;
use url::Url;
use uuid::Uuid;

//#[instrument]
pub async fn send_notifications(
    messages: Notificator,
    sender_emails_repo: &SenderEmailsRepo
) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    let mut res = 0;
    for item in messages {
        let email = item.0;
        let asset_subscribed = item.1.keys().next().unwrap().to_owned();
        let asset_similars = item
            .1
            .values()
            .next()
            .unwrap()
            .iter()
            .map(|(asset, uuid)| (asset.to_owned(), *uuid ))
            .collect::<Vec<(Url , Uuid)>>();
            //.keys()
            //.map(|asset| asset.to_owned())
            //.collect::<Vec<Url>>();

        let op =sender_emails_repo.send_new_similar_content_found(
            email,
            asset_subscribed,
            asset_similars,
         ).await;

         if let Err(sent) =op {
                log::error!("Error: Could not send email: {sent:?}");
                continue;
         }
        
        res += 1;
    }

    Ok(res)
}
