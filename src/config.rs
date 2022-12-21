use std::str::FromStr;

use actix_web::http::Uri;
use aws_config::SdkConfig;
use aws_sdk_dynamodb::{Endpoint, Region};
use dotenv::dotenv;
use env_logger::Env;
use serde::Deserialize;

pub static ENV_VAR_ENVIRONMENT: &str = "ENVIRONMENT";
pub static DEV_ENV: &str= "development";

#[derive(Deserialize, Clone, Debug)]
pub struct EnvironmentVariables {
    //pub admin_account_userid: String,
    //pub admin_account_device: String,
    pub jwt_token_base: String,
    pub environment: String,
    pub hmac_secret: String,
    pub rust_log: String,
    pub local_address: String,
    pub local_port: String,
    pub aws_region: String,
    pub aws_dynamodb_endpoint: String,
}

#[derive(Clone, Debug)]
pub struct Config {
    aws_config: Option<SdkConfig>,
    env_variables: Option<EnvironmentVariables>,
}

impl Config {
    
    pub fn new() -> Config {
        Config { aws_config: None , env_variables: None }
    }

    pub async fn setup(&mut self) {

        let check_env = std::env::var(ENV_VAR_ENVIRONMENT);//.unwrap_or_else("local");
        match check_env{
            Err(e) =>  eprintln!("Not environment variable found! {}", e),
            Ok(env)  => {
                if env== DEV_ENV {
                    dotenv().ok();
                }
            }
        }
        
        match envy::from_env::<EnvironmentVariables>() {
            Ok(env_vars) => {
                //println!("{:#?}", env_vars);
                self.env_variables = Some(env_vars.clone());
            }
            Err(error) => eprintln!("{:#?}", error),
        }

        let env = self.env_variables.as_ref().unwrap();

        let uri = Uri::from_str( env.aws_dynamodb_endpoint.as_str()  ).unwrap();
        let endpoint_resolver = Endpoint::immutable( uri  );
        let region_provider = Region::new(env.aws_region.clone());
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

    }
    pub fn aws_config(&self) -> &SdkConfig {
        let aux = self.aws_config.as_ref().unwrap();
        return aux;
    }
    pub fn env_vars(&self) -> &EnvironmentVariables {
        let aux = self.env_variables.as_ref().unwrap();
        return aux;
    }
}
