use url::Url;
use uuid::Uuid;

//#[instrument]
pub fn get_intent_message(
    email: String,
    asset: Url,
    subscription_id: Uuid,
) -> (String, String, String) {

    let subject = "Truly.video confirmation subscription".to_string();

    let body_flat_text = format!(
        r#"
        Hi {email},
        
        You've subscribed to get notified when a similar video to {asset_url} could be found!
        Please click on the following link to confirm your email subscription: https://truly.video/confirm?subscription={subscription_id}
        
        You can unsubscribe at any time by clicking on the following link: https://truly.video/unsubscribe?subscription={subscription_id}

        If you've got any doubts, please, don't hesitate to contact us by our Discord channel: https://disboard.org/server/1164515811390664735 
        We really appreciate your feedback.

        Joan from truly.video
        "#,
        email = email,
        asset_url = asset,
        subscription_id = subscription_id
    );

    let body_html = format!(
        r#"
<html>
    <head></head>
    <body>
        <p>Hi {email},</p>
        
        <p>You've subscribed to get notified when a similar video to <a href="{asset_url}">{asset_url}</a> could be found!</p>
        <p>Please click on the following link to confirm your email subscription: 
           <a href="https://truly.video/confirm?subscription={subscription_id}">Confirm Subscription</a>
        </p>

        <p>You can unsubscribe at any time by clicking on the following link: 
           <a href="https://truly.video/unsubscribe?subscription={subscription_id}">Unsubscribe</a>
        </p>

        <p>If you have any doubts, please, don't hesitate to contact us via our 
           <a href="https://disboard.org/server/1164515811390664735">Discord channel</a>. 
           We really appreciate your feedback.
        </p>

        <p>Joan from truly.video</p>
    </body>
</html>
"#,
        email = email,
        asset_url = asset,
        subscription_id = subscription_id
    );

    (subject, body_flat_text, body_html)
}
