use aws_config::SdkConfig;
use aws_sdk_dynamodb::{Endpoint, Region};
use dotenv::dotenv;
use env_logger::Env;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct EnvironmentVariables {
    pub admin_account_userid: String,
    pub jwt_token_base: String,
    pub environment: String,
    pub hmac_secret: String,
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

        let check_env = std::env::var("ENVIRONMENT");
        match check_env{
            Err(e) =>  eprintln!("Not environment variable found! {}", e),
            Ok(env)  => {
                if env=="development" {
                    dotenv().ok();
                }
            }
        }
        
        match envy::from_env::<EnvironmentVariables>() {
            Ok(env_vars) => {
                println!("{:#?}", env_vars);
                self.env_variables = Some(env_vars.clone());
            }
            Err(error) => eprintln!("{:#?}", error),
        }


        //std::env::var("ADMIN_ACCOUNT_USERID")
        //    .expect("root admin account must set at env variables");
        //std::env::var("JWT_TOKEN_BASE").expect("jwt token base isn't defined as env variables");
        //std::env::var("ENVIRONMENT ").expect("environment isn't defined as env variables");

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
    pub fn env_vars(&self) -> &EnvironmentVariables {
        let aux = self.env_variables.as_ref().unwrap();
        return aux;
    }
}
