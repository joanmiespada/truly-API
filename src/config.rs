
use env_logger::Env;
use aws_config::{meta::region::RegionProviderChain, SdkConfig};

pub struct Config {
     aws_config: Option<SdkConfig>,
}

impl Config {
    pub fn new() -> Config{
        Config { aws_config:None }
    }
    pub async fn setup(&mut self) {

        std::env::set_var("RUST_LOG", "actix_web=debug");

        let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
        let aux  = aws_config::from_env().region(region_provider).load().await;
        self.aws_config = Some(aux);

        env_logger::init_from_env(Env::default().default_filter_or("info"));

        
        //replace(&mut self.aws_config,  Some(aux));
    }
    pub fn getAWSConfig(&self) -> &SdkConfig {
        let aux = self.aws_config.as_ref().unwrap();
        return aux;
    }
}
