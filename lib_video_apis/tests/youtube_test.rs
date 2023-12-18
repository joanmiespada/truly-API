
use lib_config::{result::ResultE, environment::EnvironmentVariablesBuilder};
use lib_video_apis::{youtube::YoutubeAPIBuilder, ExternalData};


#[tokio::test]
async fn youtube_search_test() -> ResultE<()> {
    let key = dotenv::var("YOUTUBE_API_KEY").unwrap();
    let vars = EnvironmentVariablesBuilder::default()
                                        .youtube_api_key(Some(key))
                                        .build()?;
    let mut yt = YoutubeAPIBuilder::default().environment_vars(vars).build()?;

    let key_words= vec!["spain".to_string()];
    let res = yt.search(key_words,None,true).await?;

    assert_ne!(res.0.len(),0);

    // for item in res.0{
    //     println!("{}", item.to_string())
    // }

    Ok(())
}

#[tokio::test]
async fn youtube_get_category_test() -> ResultE<()> {
    let key = dotenv::var("YOUTUBE_API_KEY").unwrap();
    let vars = EnvironmentVariablesBuilder::default()
                                        .youtube_api_key(Some(key))
                                        .build()?;
    let mut yt = YoutubeAPIBuilder::default().environment_vars(vars).build()?;

    /*
    1 Film & Animation
    2 Autos & Vehicles
    10 Music 
    15 Pets & Animals
    17 Sports
    18 Short Movies
    19 Travel & Events
    20 Gaming
    21 Videoblogging
    22 People & Blogs
    23 Comedy
    24 Entertainment
    25 News & Politics
    26 Howto & Style
    27 Education
    28 Science & Technology
    30 Movies
    31 Anime/Animation
    32 Action/Adventure
    33 Classics
    34 Comedy
    35 Documentary
    36 Drama
    37 Family
    38 Foreign
    39 Horror
    40 Sci-Fi/Fantasy
    41 Thriller
    42 Shorts
    43 Shows
    44 Trailers

     */
    let category= vec!["20".to_string()];
    let res = yt.search_by_category(category,None,true).await?;

    assert_ne!(res.0.len(),0);

    // for item in res.0{
    //     println!("{}", item.to_string())
    // }

    Ok(())
}