use lib_video_apis::{ ExternalData, twitter::TwitterAPI};
use lib_config::result::ResultE;

#[tokio::test]
async fn facebook_recent_media_test() -> ResultE<()> {
    // Assume you have stored your access token in an environment variable
    let api_key = dotenv::var("TWITTER_API_KEY").unwrap();
    let api_secret = dotenv::var("TWITTER_API_SECRET").unwrap();
    let mut api = TwitterAPI::new(api_key,api_secret,None, None);

    let key_words = vec![ "messi".to_string()];

    let res = api.search(user_id, None,false).await?;

    // Check that the response contains media
    assert_ne!(res.0.len(), 0);

    for media in res.0 {
        println!("{}", media);
    }

    Ok(())
}
