use std::{ str::FromStr, env};

use aws_sdk_dynamodb::Client;
use lib_config::infra::build_local_stack_connection;
use lib_licenses::{
    models::asset::Asset,
    repositories::{
        assets::AssetRepo,
        schema_asset::create_schema_assets_all,
        schema_owners::create_schema_owners, shorter::ShorterRepo,
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
    env::set_var("ENVIRONMENT", "development");

    env_logger::builder().is_test(true).init();

    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port_ipv4(8000);

    let shared_config = build_local_stack_connection(host_port).await;
    let client = Client::new(&shared_config);

    let mut creation = create_schema_assets_all(&client).await;
    assert_that(&creation).is_ok();
    creation = create_schema_owners(&client).await;
    assert_that(&creation).is_ok();

    let mut conf = lib_config::config::Config::new();
    conf.set_aws_config(&shared_config);

    let repo_assets = AssetRepo::new(&conf);
    let repo_shorters = ShorterRepo::new(&conf);
    let service = AssetService::new(repo_assets,repo_shorters);

    let mut list_of_ids = Vec::new();
    let payload = list_of_assets_father_sons();
    for user in payload {
        let username = user.0.clone();

        let mut as1: CreatableFildsAsset; 

        match user.1 .1 {
            None => {
                as1 = CreatableFildsAsset {
                    url: user.1 .0.to_string(),
                    hash: "hash1234".to_string(),
                    license: String::from_str("gnu").unwrap(),
                    longitude: None,
                    latitude: None,
                    father: None,
                };
            }
            Some(father_url) => {
                let asset_father: Asset = service.get_by_url(&father_url).await?;
                let father_id = Some(asset_father.id().to_owned());
                as1 = CreatableFildsAsset {
                    url: user.1 .0.to_string(),
                    hash: "hash1234".to_string(),
                    license: String::from_str("gnu").unwrap(),
                    longitude: None,
                    latitude: None,
                    father: father_id,
                };
            }
        }

        let new_op = service.add(&mut as1, &username).await;
        assert_that!(&new_op).is_ok();

        let new_id = new_op.unwrap();
        list_of_ids.push(new_id);
        
    }

    for doc in list_of_ids{
        let ass =  service.get_by_id(&doc).await?;
        let url1 = ass.url().clone().unwrap();
        let url = url1.as_str();
        let res_expected: usize = match url{
            "http://1.com/asset1.png" => 2,
            "http://1.com/asset2.png" => 2,
            _ => 0
        };
        let ass_improved = service.get_by_id_enhanced(&doc).await?;

        assert_eq!( ass_improved.sons.iter().count(), res_expected );

    }


    Ok(())
}
