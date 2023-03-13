
variable "lambda_licenses_file" {
  description = "The binary zip file for the user lambda."
  type        = string
  default     = "lambda_license/bootstrap.zip"
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
variable "resource_kms" {}
variable "resource_sqs" {}
variable "resource_sns" {}
variable "role" {}
variable "environment_flag" {}
variable "trace_log" {}
variable "lambda_deploy_folder" {}

variable "blockchain_url" {
  type        = string
  description = "endpoint from our blockchain gateway"

}

variable "contract_address" {
  type        = string
  description = "hex direction where is our contract in the blockchain"

}

variable "contract_owner_address" {
  type        = string
  description = "hex direction from account user who deployed the contract in the blockchain"

}

variable "kms_cypher_owner" {
  type      = string
  description = "kms cypher for secret keys"
}

variable "blockchain_confirmations" {
  type      = number
  description = "number of confirmations from blockchain to commit a transaction"
  default = 3
}

variable "dead_letter_queue_mint" {
  type      = string
  description = "queue url to send errors when minting"
}
variable "minting_async_topic_arn" {
  type      = string
  description = "topic arn to mint async"
}

variable "rust_backtrace" {
  type= string
}

variable "video_in_topic" {
  type = string
  description = "topic to be connectec with other dependencies, video processing triggers"
}

variable "video_out_topic" {
  type = string
  desdescription = "topic to be connected with other dependencies, video processing results" 
}