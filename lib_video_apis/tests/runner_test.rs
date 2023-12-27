use std::env;

use lib_async_ops::sns::create;
use lib_config::{
    config::Config,
    environment::{DEV_ENV, ENV_VAR_ENVIRONMENT},
    infra::build_local_stack_connection,
    result::ResultE,
    schema::Schema,
};
use lib_licenses::{
    repositories::{
        assets::AssetRepo, schema_asset::AssetAllSchema, schema_owners::OwnerSchema,
        shorter::ShorterRepo,
    },
    services::{assets::AssetService, video::VideoService},
};
use lib_video_apis::{runner::RunnerBuilder, twitch::ID as TWITCH_ID, youtube::ID as YOUTUBE_ID};
use testcontainers::{clients, images};

#[tokio::test]
async fn runner_recent_media_test() -> ResultE<()> {
    let youtube_key = dotenv::var("YOUTUBE_API_KEY")?;
    let twitch_client_id = dotenv::var("TWITCH_CLIENT_ID")?;
    let twitch_client_secret = dotenv::var("TWITCH_CLIENT_SECRET")?;
    env::set_var("YOUTUBE_API_KEY", youtube_key);
    env::set_var("TWITCH_CLIENT_ID", twitch_client_id);
    env::set_var("TWITCH_CLIENT_SECRET", twitch_client_secret);
    env::set_var("RUST_LOG", "debug");
    env::set_var("AWS_REGION", "eu-central-1");
    env::set_var(ENV_VAR_ENVIRONMENT, DEV_ENV);

    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();
    let mut local_stack = images::local_stack::LocalStack::default();
    local_stack.set_services("dynamodb,sns");
    let node = docker.run(local_stack);
    let host_port = node.get_host_port_ipv4(4566);

    let shared_config = build_local_stack_connection(host_port).await;

    let mut conf = Config::new();
    conf.setup().await;
    conf.set_aws_config(&shared_config);
    let topic_arn = create(&conf, "video_in_topic".to_string()).await?;
    env::set_var("TOPIC_ARN_HASHES_SIMILARS_START", topic_arn);
    conf.refresh_env_vars();

    let creation = AssetAllSchema::create_schema(&conf).await;
    assert!(creation.is_ok());
    let creation = OwnerSchema::create_schema(&conf).await;
    assert!(creation.is_ok());

    let platform_ids = vec![YOUTUBE_ID.to_string(), TWITCH_ID.to_string()];

    let asset_repo = AssetRepo::new(&conf);
    let shorter_repo = ShorterRepo::new(&conf);
    let asset_service = AssetService::new(asset_repo, shorter_repo);
    let video_service = VideoService::new(asset_service.to_owned(), conf.to_owned());

    let r = RunnerBuilder::default()
        .environment_vars(conf.env_vars().to_owned())
        .platform_ids(platform_ids)
        .asset_service(asset_service)
        .video_service(video_service)
        .build()?;

    let word_keys = vec!["fortnite".to_string(), "playstation".to_string()];
    let res = r.process_searches(word_keys).await;

    assert_eq!(res.is_ok(), true);

    Ok(())
}
