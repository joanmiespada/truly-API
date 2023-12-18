
use lib_config::result::ResultE;
use lib_video_apis::{twitch::TwitchAPIBuilder, ExternalData};


#[tokio::test]
async fn twitch_search_test() -> ResultE<()> {
    let id = dotenv::var("TWITCH_CLIENT_ID").unwrap();
    let secret = dotenv::var("TWITCH_CLIENT_SECRET").unwrap();
    let mut tw = TwitchAPIBuilder::default().client_id(id).client_secret(secret).token(None).token_expiration(None).build()?;

    //let key_words= vec!["borderlands-3".to_string()];
    let key_words= vec!["fortnite".to_string()];
    let res = tw.search_by_category(key_words,None,true).await?;

    assert_ne!(res.0.len(),0);

    for item in res.0{
        println!("{}", item.to_string())
    }

    Ok(())
}

