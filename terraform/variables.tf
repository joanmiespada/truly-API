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

variable "service_tag" {
  description = "for tagging all resources linked to this project"
  type = string
  default = "api"
}


variable "trace_log" {
  description = "tracing lambdas"
  type = string
  //default = "cargo_lambda=info" //"cargo_lambda=trace"
}

variable "lambda_deploy_folder" {
  description = "it helps to identify the correct folder lambda with infra arm64 or linux"
  type = string
 // default = "../target/lambda_arm64/"
}

variable "kms_id_cypher_all_secret_keys"  {
  description = "kms id key where the api uses to encrypt all private keys"
  type    = string
}

variable "rust_backtrace" {
  type = string
  description = "debug info"
}

variable "jwt_token_time_exp_hours" {
  type = number
  description = "jwt token expiration time, it forces to relogin"
}

variable "dns_prefix" {
  type = string
  description = "dns for the api, staging, etc... to be concatenated with dns_base"
}
variable "dns_base" {
  type = string
  description = "domain base truly.video"
  #default = "truly.video"
} 

variable "telemetry" {
  type=bool
  description = "enable or disable telemetry"
}

variable "telemetry_endpoint" {
  type=string
  description = "endpoint to forward observability metrics"
}

variable "email" {
  description = "AWS SES email notifications"
  type    = string
}

variable "api_stage_version" {
  description = "allow to deploy multiple version: v1, v2, v3"
  type    = string
  default = "v1"
}

variable "architectures" {
  type    = list(string)
  default = [ "arm64" ]
}

variable "handler" {
  type    = string
  default = "function_handler"
}
variable "runtime" {
  type    = string
  default = "provided.al2"
}

