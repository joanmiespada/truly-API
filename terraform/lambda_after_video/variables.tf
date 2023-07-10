
variable "lambda_after_video_file" {
  description = "The binary zip file for the user lambda."
  type        = string
  default     = "lambda_after_video/bootstrap.zip"
}

variable "truly_lambda_after_video_function_name" {
  default = "truly_after_video"
  type    = string
}

variable "service_name" {
  default = "after_video_update"
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


variable "environment_flag" {}

variable "trace_log" {}

variable "lambda_deploy_folder" {}

variable "kms_cypher_owner" {
  type      = string
  description = "kms cypher for secret keys"
}
variable "rust_backtrace" {
  type= string
}
variable "sqs_after_video_process_arn" {
  type= string
}

variable "aws_region" {
  type    = string
}

variable "api_stage_version" {
  type = string
}
variable "architectures" {
  type    = list(string)
}
variable "handler" {
  type    = string
}
variable "runtime" {
  type    = string
}

