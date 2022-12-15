
use aws_config::{ SdkConfig};
use aws_sdk_dynamodb::{Endpoint, Region};
use env_logger::Env;

pub struct Config {
    aws_config: Option<SdkConfig>,
}

impl Config {
    pub fn new() -> Config {
        Config { aws_config: None }
    }
    pub async fn setup(&mut self) {
        std::env::set_var("RUST_LOG", "actix_web=debug");

        let endpoint_resolver =
            Endpoint::immutable("http://localhost:8000".parse().expect("invalid URI"));
        let region_provider = Region::new("local");
        /* 
            RegionProviderChain::first_try(env::var("local").ok().map(Region::new))
                .or_default_provider()
                .or_else(Region::new("us-east-1")); */
        let aux = aws_config::from_env()
            .region(region_provider)
            .endpoint_resolver(endpoint_resolver)
            .load()
            .await;
        self.aws_config = Some(aux);

        env_logger::init_from_env(Env::default().default_filter_or("info"));

        //replace(&mut self.aws_config,  Some(aux));
    }
    pub fn aws_config(&self) -> &SdkConfig {
        let aux = self.aws_config.as_ref().unwrap();
        return aux;
    }
}
