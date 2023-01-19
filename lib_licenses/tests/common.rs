
use aws_config::SdkConfig;
use aws_sdk_dynamodb::Credentials;
use aws_config::meta::region::RegionProviderChain;


pub async fn build_dynamodb(host_port: u16) -> SdkConfig {
    let endpoint_url = format!("http://127.0.0.1:{}", host_port);
    //let uri = Uri::from_str(&endpoint_uri).unwrap();
    //let endpoint_resolver = Endpoint::immutable_uri(uri);
    let region_provider = RegionProviderChain::default_provider().or_else("eu-central-1");
    let creds = Credentials::new("fakeKey", "fakeSecret", None, None, "test");

    let shared_config = aws_config::from_env()
        .region(region_provider)
        .endpoint_url(endpoint_url)
        //.endpoint_resolver(endpoint_resolver.unwrap())
        .credentials_provider(creds)
        .load()
        .await;

    //Client::new(&shared_config)
    return shared_config;
}