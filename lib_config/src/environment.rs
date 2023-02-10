
use serde::Deserialize;

pub static ENV_VAR_ENVIRONMENT: &str = "ENVIRONMENT";
pub static DEV_ENV: &str = "development";
pub static PROD_ENV: &str = "production";
pub static STAGE_ENV: &str = "stage";

#[derive(Deserialize, Clone, Debug)]
pub struct EnvironmentVariables {
    jwt_token_base: Option<String>,
    jwt_token_time_exp_hours: Option<String>,
    environment: String,
    hmac_secret: Option<String>,
    rust_log: String,
    aws_region: Option<String>,
    aws_endpoint: Option<String>,

    blockchain_url: Option<String>,
    contract_address: Option<String>,
    contract_owner_address: Option<String>,

    kms_key_id: Option<String>,
    blockchain_confirmations: Option<usize>,
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
    
    pub fn set_hmac_secret(&mut self, value: String) {
        self.hmac_secret = Some(value.clone());
    }

    pub fn jwt_token_base(&self) -> &String {
        let aux = self.jwt_token_base.as_ref().unwrap();
        return aux;
    }
    
    pub fn set_jwt_token_base(&mut self, value: String) {
        self.jwt_token_base = Some(value.clone());
    }


    pub fn aws_region(&self) -> &String {
        let aux = self.aws_region.as_ref().unwrap();
        return aux;
    }
    pub fn aws_endpoint(&self) -> &String {
        let aux = self.aws_endpoint.as_ref().unwrap();
        return aux;
    }

    pub fn jwt_token_time_exp_hours(&self) -> &String {
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

