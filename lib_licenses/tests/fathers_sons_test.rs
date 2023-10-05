use std::{env, str::FromStr};

use lib_config::{
    config::Config,
    environment::{DEV_ENV, ENV_VAR_ENVIRONMENT},
    infra::build_local_stack_connection,
    schema::Schema,
};
use lib_licenses::{
    models::asset::{Asset, SourceType},
    repositories::{
        assets::AssetRepo, schema_asset::AssetAllSchema, schema_owners::OwnerSchema,
        shorter::ShorterRepo,
    },
    services::assets::{AssetManipulation, AssetService, CreatableFildsAsset},
};
use spectral::{assert_that, result::ResultAssertions};
use testcontainers::*;
use url::Url;

fn list_of_assets_father_sons() -> Vec<(String, (Url, Option<Url>))> {
    let mut aux = Vec::new();

    aux.push((
        "user1".to_string(),
        (Url::parse("http://1.com/asset1.png").unwrap(), None),
    ));
    aux.push((
        "user2".to_string(),
        (Url::parse("http://1.com/asset2.png").unwrap(), None),
    ));

    aux.push((
        "user1".to_string(),
        (
            Url::parse("http://1.com/asset3.png").unwrap(),
            Some(Url::parse("http://1.com/asset1.png").unwrap()),
        ),
    ));

    aux.push((
        "user1".to_string(),
        (
            Url::parse("http://1.com/asset4.png").unwrap(),
            Some(Url::parse("http://1.com/asset1.png").unwrap()),
        ),
    ));

    aux.push((
        "user2".to_string(),
        (
            Url::parse("http://1.com/asset5.png").unwrap(),
            Some(Url::parse("http://1.com/asset2.png").unwrap()),
        ),
    ));
    aux.push((
        "user2".to_string(),
        (
            Url::parse("http://1.com/asset6.png").unwrap(),
            Some(Url::parse("http://1.com/asset2.png").unwrap()),
        ),
    ));
    return aux;
}

#[tokio::test]
async fn check_asset_sons() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env::set_var("RUST_LOG", "debug");
    env::set_var(ENV_VAR_ENVIRONMENT, DEV_ENV);

    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config = build_local_stack_connection(host_port).await;

    let mut conf = Config::new();
    conf.setup().await;
    conf.set_aws_config(&shared_config);

    let creation = AssetAllSchema::create_schema(&conf).await;
    assert_that(&creation).is_ok();
    let creation = OwnerSchema::create_schema(&conf).await;
    assert_that(&creation).is_ok();

    let repo_assets = AssetRepo::new(&conf);
    let repo_shorters = ShorterRepo::new(&conf);
    let service = AssetService::new(repo_assets, repo_shorters);

    let mut list_of_ids = Vec::new();
    let payload = list_of_assets_father_sons();
    for user in payload {
        let username = user.0.clone();

        let mut as1: CreatableFildsAsset;

        match user.1 .1 {
            None => {
                as1 = CreatableFildsAsset {
                    url: user.1 .0.to_string(),
                    hash: Some("hash1234".to_string()),
                    hash_algorithm: Some("MD5".to_string()),
                    license: Some(String::from_str("gnu").unwrap()),
                    longitude: None,
                    latitude: None,
                    father: None,
                    source: SourceType::Others,
                    source_details: None,
                };
            }
            Some(father_url) => {
                let asset_father: Asset = service.get_by_url(&father_url).await?;
                let father_id = Some(asset_father.id().to_owned());
                as1 = CreatableFildsAsset {
                    url: user.1 .0.to_string(),
                    hash: Some("hash1234".to_string()),
                    hash_algorithm: Some("MD5".to_string()),
                    license: Some(String::from_str("gnu").unwrap()),
                    longitude: None,
                    latitude: None,
                    father: father_id,
                    source: SourceType::Others,
                    source_details: None,
                };
            }
        }

        let new_op = service.add(&mut as1, &Some(username)).await;
        assert_that!(&new_op).is_ok();

        let new_id = new_op.unwrap();
        list_of_ids.push(new_id);
    }

    for doc in list_of_ids {
        let ass = service.get_by_id(&doc).await?;
        let url1 = ass.url().clone().unwrap();
        let url = url1.as_str();
        let res_expected: usize = match url {
            "http://1.com/asset1.png" => 2,
            "http://1.com/asset2.png" => 2,
            _ => 0,
        };
        let ass_improved = service.get_by_id_enhanced(&doc).await?;

        assert_eq!(ass_improved.sons.iter().count(), res_expected);
    }

    Ok(())
}
