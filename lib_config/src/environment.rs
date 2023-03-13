
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
    blockchain_gateway_api_key: Option<String>,
    contract_address: Option<String>,
    contract_owner_address: Option<String>,

    kms_key_id: Option<String>,
    blockchain_confirmations: Option<usize>,
    queue_mint_async: Option<String>,
    topic_arn_mint_async: Option<String>,
    dead_letter_queue_mint: Option<String>,
    
    shorter_video_in_topic: Option<String>,
    shorter_video_out_topic: Option<String>,

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
    pub fn queue_mint_async (&self) -> &String {
        let aux = self.queue_mint_async.as_ref().unwrap();
        return aux;
    }
    pub fn set_queue_mint_async(&mut self, value: String) {
        self.queue_mint_async = Some(value.clone());
    }
    pub fn dead_letter_queue_mint (&self) -> &String {
        let aux = self.dead_letter_queue_mint.as_ref().unwrap();
        return aux;
    }
    pub fn set_dead_letter_queue_mint(&mut self, value: String) {
        self.dead_letter_queue_mint = Some(value.clone());
    }

    pub fn topic_arn_mint_async(&self)-> &String {
        let aux = self.topic_arn_mint_async.as_ref().unwrap();
        return aux;
    }
    pub fn set_topic_arn_mint_async(&mut self, value: String) {
        self.topic_arn_mint_async = Some(value.clone());
    }

    pub fn blockchain_gateway_api_key(&self)-> &String{
        self.blockchain_gateway_api_key.as_ref().unwrap()
    }
    pub fn set_blockchain_gateway_api_key(&mut self, value: String) {
        self.blockchain_gateway_api_key = Some(value.clone());
    }

    pub fn topic_arn_shorter_video_start(&self)-> &String{
        self.shorter_video_in_topic.as_ref().unwrap()
    }
    pub fn set_topic_arn_shorter_video_start(&mut self, value: String) {
        self.shorter_video_in_topic= Some(value.clone());
    }

    pub fn topic_arn_shorter_video_result(&self)-> &String{
        self.shorter_video_out_topic.as_ref().unwrap()
    }
    pub fn set_topic_arn_shorter_video_result(&mut self, value: String) {
        self.shorter_video_out_topic= Some(value.clone());
    }


}

