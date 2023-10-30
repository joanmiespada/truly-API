use url::Url;
use uuid::Uuid;

//#[instrument]
pub fn get_similar_content_found_message(
    subscription_id: Option<Uuid>,
    email: String,
    asset_subscribed: Url,
    asset_similars: Vec<Url>,
) -> (String, String, String) {
    let asset_similars_text = asset_similars
        .iter()
        .map(|asset| format!(" - {}\n", asset))
        .collect::<String>();

    let subject = "Truly.video new videos found".to_string();

    let body_flat_text = format!(
        r#"
Hi {email},

We have found similar videos to your video: {asset_subscribed}
Please review: 

{asset_similars}

If you've got any doubts, please, don't hesitate to contact us by our Discord channel: https://disboard.org/server/1164515811390664735 
We really appreciate your feedback.


Joan from truly.video
"#,
        email = email,
        asset_subscribed = asset_subscribed,
        asset_similars = asset_similars_text,
        //subscription_id = subscription_id
    );

//You can unsubscribe at any time by clicking on the following link: https://truly.video/unsubscribe?subscription={subscription_id}

    let asset_similars_html = asset_similars
        .iter()
        .map(|asset| format!("<li>{}</li>", asset))
        .collect::<String>();

    let body_html = format!(
        r#"
<p>Hi {email}</p>,

<p>We have found similar videos to your video subscription: {asset_subscribed}.</p>
<p>Please review: </p>
<ul>
{asset_similars}
</ul>
<p>If you've got any doubts, please, don't hesitate to contact us by our Discord channel: <a href="https://disboard.org/server/1164515811390664735">Discord channel</a>. 
We really appreciate your feedback.</p>


Joan from truly.video
"#,
        email = email,
        asset_subscribed = asset_subscribed,
        asset_similars = asset_similars_html
    );
// <p>You can unsubscribe at any time by clicking on the following link: 
//     <a href="https://truly.video/unsubscribe?subscription={subscription_id}">Unsubscribe</a>
// </p>
    (subject, body_flat_text, body_html)
}
