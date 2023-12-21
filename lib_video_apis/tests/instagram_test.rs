use lib_video_apis::{instagram::InstagramAPI, ExternalData};
use lib_config::result::ResultE;

#[tokio::test]
async fn instagram_recent_media_test() -> ResultE<()> {
    // Assume you have stored your access token in an environment variable
    let access_token = dotenv::var("INSTAGRAM_ACCESS_TOKEN").unwrap();
    let mut ig_api = InstagramAPI::new(Some(access_token), None);

    // Define the user ID for which you want to retrieve recent media
    // In a real scenario, this would be obtained through an OAuth flow
    let user_id = vec![ "user_id_here".to_string()];

    let res = ig_api.search(user_id, None,false).await?;

    // Check that the response contains media
    assert_ne!(res.0.len(), 0);

    for media in res.0 {
        println!("{}", media);
    }

    Ok(())
}
