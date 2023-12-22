use lib_video_apis::{facebook::FacebookAPI, ExternalData};
use lib_config::result::ResultE;

#[tokio::test]
async fn facebook_recent_media_test() -> ResultE<()> {
    // Assume you have stored your access token in an environment variable
    let access_token = dotenv::var("FACEBOOK_ACCESS_TOKEN").unwrap();
    let mut ig_api = FacebookAPI::new(Some(access_token), None);

    let tags = vec![ "london".to_string()];

    let res = ig_api.search(tags, None,false).await?;

    // Check that the response contains media
    assert_ne!(res.0.len(), 0);

    for media in res.0 {
        println!("{}", media);
    }

    Ok(())
}
