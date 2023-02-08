
use aws_config::{meta::region::RegionProviderChain, SdkConfig};
use dotenv::dotenv;
use serde::Deserialize;

pub static ENV_VAR_ENVIRONMENT: &str = "ENVIRONMENT";
pub static DEV_ENV: &str = "development";
pub static PROD_ENV: &str = "production";

pub const SECRETS_MANAGER_KEYS: &str = "truly/api/secrets";
pub const SECRETS_MANAGER_SECRET_KEY: &str = "truly/api/secret_key";

#[derive(Deserialize, Clone, Debug)]
pub struct EnvironmentVariables {
    jwt_token_base: Option<String>,
    jwt_token_time_exp_hours : Option<String>, 
    environment: String,
    hmac_secret: Option<String>,
    rust_log: String,
    aws_region: Option<String>,
    aws_endpoint: Option<String>,

    blockchain_url: Option<String>,
    contract_address: Option<String>,
    contract_owner_address: Option<String>,

    kms_key_id: Option<String>,
    blockchain_confirmations: Option<usize>
}

impl EnvironmentVariables {

    pub fn rust_log(&self) -> &String {
        let aux = &self.rust_log;
        return aux;
    }
    
    pub fn environment(&self) -> &String {
        let aux = &self.environment;
        return aux;
    }

    pub fn hmac_secret(&self) -> &String {
        let aux = self.hmac_secret.as_ref().unwrap();
        return aux;
    }
    pub fn jwt_token_base(&self) -> &String {
        let aux = self.jwt_token_base.as_ref().unwrap();
        return aux;
    }
    pub fn aws_region(&self) -> &String {
        let aux = self.aws_region.as_ref().unwrap();
        return aux;
    }
    pub fn aws_endpoint(&self) -> &String {
        let aux = self.aws_endpoint.as_ref().unwrap();
        return aux;
    }

    pub fn jwt_token_time_exp_hours (&self) -> &String {
        let aux = self.jwt_token_time_exp_hours.as_ref().unwrap();
        return aux;
    }

    pub fn blockchain_url(&self) -> &String {
        let aux = self.blockchain_url.as_ref().unwrap();
        return aux;
    }
    
    pub fn set_blockchain_url(&mut self, new_url: String) {
        self.blockchain_url = Some(new_url.clone());
    }
    
    pub fn contract_address(&self) -> &String {
        let aux = self.contract_address.as_ref().unwrap();
        return aux;
    }
    pub fn set_contract_address(&mut self, new_addres: String) {
        self.contract_address = Some(new_addres.clone());
    }

    pub fn contract_owner_address(&self) -> &String {
        let aux = self.contract_owner_address.as_ref().unwrap();
        return aux;
    }
    pub fn set_contract_owner_address(&mut self, value: String) {
        self.contract_owner_address = Some(value.clone());
    }
    
    pub fn kms_key_id(&self) -> &String {
        let aux = self.kms_key_id.as_ref().unwrap();
        return aux;
    }
    pub fn set_kms_key_id(&mut self, value: String) {
        self.kms_key_id = Some(value.clone());
    }
    
    pub fn blockchain_confirmations(&self) -> &usize {
        let aux = self.blockchain_confirmations.as_ref().unwrap();
        return aux;
    }
    pub fn set_blockchain_confirmations(&mut self, value: usize) {
        self.blockchain_confirmations = Some(value.clone());
    }

}

#[derive(Clone, Debug)]
pub struct Config {
    aws_config: Option<SdkConfig>,
    env_variables: Option<EnvironmentVariables>,
}

#[derive(Deserialize, Debug)]
struct Secrets {
    #[serde(rename = "HMAC_SECRET")]
    hmac_secret: String,
    #[serde(rename = "JWT_TOKEN_BASE")]
    jwt_token_base: String,

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
                if env == DEV_ENV {
                    dotenv().ok();
                }
            }
        }

        match envy::from_env::<EnvironmentVariables>() {
            Ok(env_vars) => {
                self.env_variables = Some(env_vars.clone());
            }
            Err(error) => panic!("Environment variables are missing {:#?}", error),
        }

        let env = self.env_variables.as_ref().unwrap();
        let config: SdkConfig;
        if env.environment == DEV_ENV {
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
        } else if env.environment == PROD_ENV {
            let region_provider = RegionProviderChain::default_provider().or_else("eu-central-1");
            config = aws_config::from_env().region(region_provider).load().await;
        } else {
            panic!(
                "environment variable ENVIRONMENT configured wrongly: {}",
                env.environment
            )
        }
        self.aws_config = Some(config);

    }


    pub fn aws_config(&self) -> &SdkConfig {
        let aux = self.aws_config.as_ref().unwrap();
        return aux;
    }
    pub fn set_aws_config(&mut self, cnf: &SdkConfig ) {
        //let aux = self.aws_config.as_ref().unwrap();
        //return aux;
        self.aws_config = Some(cnf.clone());
    }
    pub fn env_vars(&self) -> &EnvironmentVariables {
        let aux = self.env_variables.as_ref().unwrap();
        return aux;
    }
    pub fn set_env_vars(&mut self, new_data: &EnvironmentVariables ) {
        self.env_variables = Some(new_data.clone() )
    }

    async fn load_secrets(&mut self) {
        let client = aws_sdk_secretsmanager::Client::new(self.aws_config());
        let mut resp = client
            .get_secret_value()
            .secret_id(SECRETS_MANAGER_KEYS)
            .send()
            .await;

        match resp {
            Err(e) => {
                panic!("secrets couldn't find: {}", e.to_string())
            }
            Ok(scr) => {
                let value = scr.secret_string().unwrap();
                let mut m_env = self.env_variables.as_mut().unwrap();
                let secrets: Secrets = serde_json::from_str(value).unwrap(); //_or( panic!("secrets malformed") );
                m_env.hmac_secret = Some(secrets.hmac_secret);
                m_env.jwt_token_base = Some(secrets.jwt_token_base);
            }
        }
        //check secret key is stored
        resp = client
            .get_secret_value()
            .secret_id( SECRETS_MANAGER_SECRET_KEY)
            .send()
            .await;

        match resp {
            Err(e) => {
                panic!("secret key for contract owner couldn't find: {}", e.to_string())
            }
            Ok(scr) => {
                let _value = scr.secret_string().unwrap();
            }
        }

    }

}
