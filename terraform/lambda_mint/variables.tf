
variable "lambda_mint_file" {
  description = "The binary zip file for the user lambda."
  type        = string
  default     = "lambda_mint/bootstrap.zip"
}

variable "truly_lambda_mint_function_name" {
  default = "truly_mint"
  type    = string
}

variable "service_name" {
  default = "minting"
  type    = string
}

variable "common_tags" {}
variable "resource_logs" {}
variable "resource_dynamodb" {}
variable "resource_xray" {}
variable "resource_secretsman" {}
variable "resource_kms" {}
variable "resource_sqs" {}

variable "role" {}

variable "regions" {}

variable "environment_flag" {}

variable "trace_log" {}

variable "lambda_deploy_folder" {}

variable "kms_cypher_owner" {
  type      = string
  description = "kms cypher for secret keys"
}

variable "dead_letter_queue_mint" {
  //type      = string
  type = map(any)
  description = "queue url to send errors when minting"
}

variable "queue_mint_arn"{
  //type = string
  type = map(any)
  description = "when this queue recieves a message this lamdbda will get up and process it"
}

variable "rust_backtrace" {
  type= string
}

variable "minting_async_topic_arn" {
  //type      = string
  type = map(any)
  description = "topic arn to mint async"
}

variable minting_fails_topic_arn {
  //type = string
  type = map(any)
  description = "topic to register when miting fails after several retries"
}

variable "function_handler" {}
variable "runtime" {}
variable "architecture" {}