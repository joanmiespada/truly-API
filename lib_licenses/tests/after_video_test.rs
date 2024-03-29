use std::env;
use std::str::FromStr;

use lib_async_ops::sns::create;
use lib_config::config::Config;
use lib_config::environment::{DEV_ENV, ENV_VAR_ENVIRONMENT};
use lib_config::infra::build_local_stack_connection;
use lib_config::schema::Schema;
use lib_licenses::models::asset::VideoLicensingStatus;
use lib_licenses::repositories::assets::AssetRepo;
use lib_licenses::repositories::schema_asset::AssetAllSchema;
use lib_licenses::repositories::schema_owners::OwnerSchema;
use lib_licenses::repositories::shorter::ShorterRepo;
use lib_licenses::services::assets::{AssetManipulation, AssetService, CreatableFildsAsset};
use lib_licenses::services::video::{VideoManipulation, VideoService};
use lib_video_objs::shorter::CreateShorter;
use lib_video_objs::video::{VideoProcessStatus, VideoResult};
use testcontainers::*;
use url::Url;
use uuid::Uuid;

#[tokio::test]
async fn add_after_video_process() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    //env::set_var("SHORTER_VIDEO_IN_TOPIC", topic_arn);
    conf.refresh_env_vars();

    let creation = AssetAllSchema::create_schema(&conf).await;
    assert!(creation.is_ok());
    let creation = OwnerSchema::create_schema(&conf).await;
    assert!(creation.is_ok());

    let repo_assets = AssetRepo::new(&conf);
    let repo_shorters = ShorterRepo::new(&conf);
    let service = AssetService::new(repo_assets, repo_shorters);

    let video_service = VideoService::new(service.to_owned(), conf.to_owned());

    let user_id = Uuid::new_v4().to_string();

    let creation_asset = CreatableFildsAsset {
        license: Some("gnu".to_string()),
        latitude: None,
        longitude: None,
        url: "http://w111.test.com/f1.mov".to_string(),
        hash: Some("hash_f1".to_string()),
        hash_algorithm: Some("MD5".to_string()),
        father: None,
        source: None, // Some(SourceType::Others),
        source_details: None,
    };
    let shorter_id = "0".to_string();

    let asset_original_op = service.add(&creation_asset, &Some(user_id.clone())).await;
    assert!(asset_original_op.is_ok());
    let asset_original = asset_original_op.unwrap();

    let shorter = CreateShorter {
        url_file: Url::from_str(creation_asset.url.as_str()).unwrap(),
        hash: creation_asset.hash.to_owned().unwrap(),
        hash_algorithm: creation_asset.hash_algorithm.to_owned().unwrap(),
        user_id: user_id.to_owned(),
        asset_id: asset_original,
        keep_original: true,
    };

    let post_request_op = video_service
        .shorter_video_async(&asset_original, &user_id)
        .await;
    assert!(post_request_op.is_ok());

    //simulate response after posting the request:
    let mut video_res = VideoResult {
        url_file: Url::from_str(creation_asset.url.as_str()).unwrap(),
        hash: creation_asset.hash.to_owned().unwrap(),
        hash_algorithm: creation_asset.hash_algorithm.to_owned().unwrap(),
        user_id: user_id.to_owned(),
        asset_id: asset_original,
        keep_original: shorter.keep_original,
        counter: 0,
        shorter: shorter_id.to_owned(),
        video_op: None,
        video_error: None,
        video_original: None, // Some(Url::from_str("http://w222.test.com/f1.mov").unwrap()),
        video_original_hash: None, //  Some("hash_f1".to_string()),
        video_original_hash_algorithm: None,
        video_licensed_asset_id: None, //Some(Uuid::new_v4()),
        video_licensed: None, //Some(Url::from_str("http://w222.test.com/f2.mov").unwrap()),
        video_licensed_hash: None, // Some("hash_f2".to_string()),
        video_licensed_hash_algorithm: None,
        video_process_status: Some(VideoProcessStatus::Started),
    };

    let new_op = service.store_video_process(&video_res).await;
    assert!(new_op.is_ok());

    let mut old_asset_op = service.get_by_id(&asset_original).await;
    assert!(old_asset_op.is_ok());
    let mut old_asset_father = old_asset_op.unwrap();
    assert_eq!(
        old_asset_father.video_process_status().clone().unwrap(),
        VideoProcessStatus::Started
    );

    video_res = VideoResult {
        url_file: Url::from_str(creation_asset.url.as_str()).unwrap(),
        hash: creation_asset.hash.to_owned().unwrap(),
        hash_algorithm: creation_asset.hash_algorithm.to_owned().unwrap(),
        user_id: user_id.to_owned(),
        asset_id: asset_original,
        keep_original: shorter.keep_original,
        counter: 0,
        shorter: shorter_id.to_owned(),
        video_op: None,
        video_error: None,
        video_original: None, // Some(Url::from_str("http://w222.test.com/f1.mov").unwrap()),
        video_original_hash: None, //  Some("hash_f1".to_string()),
        video_original_hash_algorithm: None,
        video_licensed_asset_id: None, //Some(Uuid::new_v4()),
        video_licensed: None, //Some(Url::from_str("http://w222.test.com/f2.mov").unwrap()),
        video_licensed_hash: None, // Some("hash_f2".to_string()),
        video_licensed_hash_algorithm: None,
        video_process_status: Some(VideoProcessStatus::Downloaded),
    };

    let new_op = service.store_video_process(&video_res).await;
    assert!(new_op.is_ok());

    old_asset_op = service.get_by_id(&asset_original).await;
    assert!(old_asset_op.is_ok());
    old_asset_father = old_asset_op.unwrap();
    assert_eq!(
        old_asset_father.video_process_status().clone().unwrap(),
        VideoProcessStatus::Downloaded
    );

    video_res = VideoResult {
        url_file: Url::from_str(creation_asset.url.as_str()).unwrap(),
        hash: creation_asset.hash.to_owned().unwrap(),
        hash_algorithm: creation_asset.hash_algorithm.to_owned().unwrap(),
        user_id,
        asset_id: asset_original,
        keep_original: shorter.keep_original,
        counter: 0,
        shorter: shorter_id.to_owned(),
        video_op: Some(true),
        video_error: None,
        video_original: Some(Url::from_str("http://w222.test.com/f1.mov").unwrap()),
        video_original_hash: Some("hash_f1".to_string()),
        video_original_hash_algorithm: Some("MD5".to_string()),
        video_licensed_asset_id: Some(Uuid::new_v4()),
        video_licensed: Some(Url::from_str("http://w222.test.com/f2.mov").unwrap()),
        video_licensed_hash: Some("hash_f2".to_string()),
        video_licensed_hash_algorithm: Some("SHA256".to_string()),
        video_process_status: Some(VideoProcessStatus::CompletedSuccessfully),
    };

    let new_op = service.store_video_process(&video_res).await;
    assert!(new_op.is_ok());

    let new_asset_op = service
        .get_by_id(&video_res.video_licensed_asset_id.unwrap())
        .await;
    assert!(new_asset_op.is_ok());

    let new_asset_son = new_asset_op.unwrap();

    assert_eq!(new_asset_son.father().unwrap(), asset_original);
    assert_eq!(
        *new_asset_son.url().as_ref().unwrap(),
        video_res.video_licensed.unwrap()
    );
    assert_eq!(
        *new_asset_son.hash().as_ref().unwrap(),
        video_res.video_licensed_hash.unwrap()
    );
    assert_eq!(
        *new_asset_son.video_licensing_status(),
        VideoLicensingStatus::AlreadyLicensed
    );

    old_asset_op = service.get_by_id(&asset_original).await;
    assert!(old_asset_op.is_ok());
    old_asset_father = old_asset_op.unwrap();

    assert_eq!(
        *old_asset_father.url().as_ref().unwrap(),
        video_res.video_original.unwrap()
    );
    assert_eq!(
        *old_asset_father.hash().as_ref().unwrap(),
        video_res.video_original_hash.unwrap()
    );
    assert_eq!(
        *old_asset_father.video_licensing_status(),
        VideoLicensingStatus::CompletedSuccessfully
    );
    assert_eq!(
        old_asset_father.video_process_status().clone().unwrap(),
        VideoProcessStatus::CompletedSuccessfully
    );

    let ass_short = service.get_by_shorter(&shorter_id).await?;
    assert_eq!(video_res.video_licensed_asset_id.unwrap(), *ass_short.id());

    Ok(())
}
