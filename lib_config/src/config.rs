use aws_config::{meta::region::RegionProviderChain, SdkConfig};
use aws_types::region::Region;
use dotenv::dotenv;
use log::{debug, info};

use crate::{
    environment::{EnvironmentVariables, DEV_ENV, ENV_VAR_ENVIRONMENT, PROD_ENV, STAGE_ENV},
    secrets::{Secrets, SECRETS_MANAGER_APP_KEYS, SECRETS_MANAGER_SECRET_KEY},
};

#[derive(Clone, Debug)]
pub struct Config {
    aws_config: Option<SdkConfig>,
    env_variables: Option<EnvironmentVariables>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            aws_config: None,
            env_variables: None,
        }
    }

    pub async fn setup_with_secrets(&mut self) {
        self._setup_basic().await;
        self.load_secrets().await;
    }
    pub async fn setup(&mut self) {
        self._setup_basic().await;
    }
    async fn _setup_basic(&mut self) {
        let check_env = std::env::var(ENV_VAR_ENVIRONMENT);
        match check_env {
            Err(e) => panic!(
                "Not environment variable found! {}. Error: {}",
                ENV_VAR_ENVIRONMENT,
                e.to_string()
            ),
            Ok(env) => {
                info!("environment var: {}", env);
                if env == DEV_ENV {
                    debug!("loading env vars from .env file");
                    dotenv().ok();
                }
            }
        }

        match envy::from_env::<EnvironmentVariables>() {
            Ok(env_vars) => {
                self.env_variables = Some(env_vars.clone());
            }
            Err(error) => panic!(
                "some mandatory environment variables are missing {:#?}",
                error
            ),
        }

        let env = self.env_variables.as_ref().unwrap();
        let config: SdkConfig;
        if env.environment() == DEV_ENV {
            //let uri = Uri::from_str(env.aws_endpoint()).unwrap();
            //let endpoint_resolver = aws_sdk_dynamodb::Endpoint::immutable_uri(uri);
            let region_provider = aws_sdk_dynamodb::Region::new(env.aws_region().clone());
            /*
            RegionProviderChain::first_try(env::var("local").ok().map(Region::new))
                .or_default_provider()
                .or_else(Region::new("us-east-1")); */
            let creden = aws_config::profile::ProfileFileCredentialsProvider::builder()
                .profile_name("localstack");
            config = aws_config::from_env()
                .credentials_provider(creden.build())
                .region(region_provider)
                .endpoint_url(env.aws_endpoint().clone())
                //.endpoint_resolver(endpoint_resolver.unwrap())
                .load()
                .await;
        } else if env.environment() == PROD_ENV {
            let region_provider = RegionProviderChain::default_provider().or_else("eu-central-1");
            config = aws_config::from_env().region(region_provider).load().await;
        } else if env.environment() == STAGE_ENV {
            let region_provider = RegionProviderChain::first_try(Region::new("eu-west-1"));
            //let region_provider = RegionProviderChain::first_try(provider) default_provider().or_else("eu-west-1");
            config = aws_config::from_env().region(region_provider).load().await;
        } else {
            panic!(
                "environment variable ENVIRONMENT configured wrongly: {}",
                env.environment()
            )
        }
        self.aws_config = Some(config);
    }

    pub fn aws_config(&self) -> &SdkConfig {
        let aux = self.aws_config.as_ref().unwrap();
        return aux;
    }
    pub fn set_aws_config(&mut self, cnf: &SdkConfig) {
        self.aws_config = Some(cnf.clone());
    }
    pub fn env_vars(&self) -> &EnvironmentVariables {
        let aux = self.env_variables.as_ref().unwrap();
        return aux;
    }
    pub fn set_env_vars(&mut self, new_data: &EnvironmentVariables) {
        self.env_variables = Some(new_data.clone())
    }

    pub async fn load_secret(&mut self, secret_id: &str) {
        let client = aws_sdk_secretsmanager::Client::new(self.aws_config());
        match secret_id {
            SECRETS_MANAGER_APP_KEYS => {
                let resp = client
                    .get_secret_value()
                    .secret_id(SECRETS_MANAGER_APP_KEYS)
                    .send()
                    .await;

                match resp {
                    Err(e) => {
                        panic!("secrets couldn't find: {}", e.to_string())
                    }
                    Ok(scr) => {
                        let value = scr.secret_string().unwrap();
                        let m_env = self.env_variables.as_mut().unwrap();
                        let secrets: Secrets = serde_json::from_str(value).unwrap(); //_or( panic!("secrets malformed") );
                        m_env.set_hmac_secret(secrets.hmac_secret);
                        m_env.set_jwt_token_base(secrets.jwt_token_base);
                        m_env.set_blockchain_gateway_api_key(secrets.blockchain_gateway_api_key);

                        debug!("app secretes found correctly")
                    }
                }
            }
            SECRETS_MANAGER_SECRET_KEY => {
                //check secret key is stored and available, but don't stored in memory!
                let resp = client
                    .get_secret_value()
                    .secret_id(SECRETS_MANAGER_SECRET_KEY)
                    .send()
                    .await;

                match resp {
                    Err(e) => {
                        panic!(
                            "secret key for contract owner couldn't find: {}",
                            e.to_string()
                        )
                    }
                    Ok(scr) => {
                        debug!("secret key found correctly!");
                        let _value = scr.secret_string().unwrap();
                    }
                }
            }
            _ => {
                panic!("secret code {} not found", secret_id)
            }
        }
    }

    pub async fn load_secrets(&mut self) {
        self.load_secret(SECRETS_MANAGER_APP_KEYS).await;
        self.load_secret(SECRETS_MANAGER_SECRET_KEY).await;
    }
}
