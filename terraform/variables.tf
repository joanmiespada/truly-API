variable "aws_region" {
  description = "AWS region for all resources."
  type    = string
}

variable "secrets_manager_app_keys_name" {
  description = "secret's manager for app values like API keys, jwt token, etc..."
  type    = string
  default = "truly_app_keys" //from lib_config::secrets
}

variable "secrets_manager_contract_owner_secret_key_name" {
  description = "secret's manager where the secret_key is stored encrypted"
  type    = string
  default = "truly_contract_owners_secret_key" //from lib_config::secrets
}

variable "environment_flag" {
  description = "environment flag"
  type    = string
}

variable "truly_tag" {
  description = "for tagging all resources linked to this project"
  type = string
  default = "truly"
}

variable "trace_log" {
  description = "tracing lambdas"
  type = string
  default = "cargo_lambda=trace"
}

variable "lambda_deploy_folder" {
  description = "it helps to identify the correct folder lambda with infra arm64 or linux"
  type = string
  default = "../target/lambda_arm64/"
}

variable "contract_owner_address" {
  description = "contract owner address"
  type    = string
}
variable "contract_address" {
  description = "address where the contract has been deployed at blockchain network"
  type    = string
}
variable "kms_id_cypher_all_secret_keys"  {
  description = "kms id key where the api uses to encrypt all private keys"
  type    = string
}
variable "blockchain_url" {
  description = "address where the contract has been deployed at blockchain network"
  type    = string
}

variable "rust_backtrace" {
  type = string
  description = "debug info"
}