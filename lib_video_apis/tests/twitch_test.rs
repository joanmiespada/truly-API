
use lib_config::{result::ResultE, environment::EnvironmentVariablesBuilder};
use lib_video_apis::{twitch::TwitchAPIBuilder, ExternalData};


#[tokio::test]
async fn twitch_search_test() -> ResultE<()> {
    let id = dotenv::var("TWITCH_CLIENT_ID").unwrap();
    let secret = dotenv::var("TWITCH_CLIENT_SECRET").unwrap();

    let vars = EnvironmentVariablesBuilder::default()
                                        .twitch_client_id(Some(id))
                                        .twitch_client_secret(Some(secret))
                                        .build()?;
    let mut tw = TwitchAPIBuilder::default().environment_vars(vars).build()?;


    //let key_words= vec!["borderlands-3".to_string()];
    let key_words= vec!["fortnite".to_string()];
    let res = tw.search_by_category(key_words,None,true).await?;

    assert_ne!(res.0.len(),0);

    for item in res.0{
        println!("{}", item.to_string())
    }

    Ok(())
}

