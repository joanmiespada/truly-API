
variable "lambda_licenses_file" {
  description = "The binary zip file for the user lambda."
  type        = string
  default     = "lambda_user/bootstrap.zip"
}

variable "truly_lambda_licenses_function_name" {
  default = "truly_licenses"
  type    = string
}

variable "service_name" {
  default = "licenses"
  type    = string
}

variable "common_tags" {}
variable "resource_logs" {}
variable "resource_dynamodb" {}
variable "resource_xray" {}
variable "resource_secretsman" {}

variable "role" {}


variable "environment_flag" {}

variable "trace_log" {}

variable "lambda_deploy_folder" {}

variable "blockchain_url" {
  type        = string
  default     = "https://localhost:1234"
  description = "endpoint from our blockchain gateway"

}

variable "contract_address" {
  type        = string
  default     = ""
  description = "hex direction where is our contract in the blockchain"

}

variable "contract_owner" {
  type        = string
  default     = ""
  description = "hex direction from account user who deployed the contract in the blockchain"

}

variable "kms_cypher_owner" {
  type      = string
  default = "2d460536-1dc9-436c-a97b-0bad3f8906c7"
  description = "kms cypher for secret keys"
}
