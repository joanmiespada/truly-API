
variable "lambda_licenses_file" {
  description = "The binary zip file for the user lambda."
  type    = string
  default = "lambda_user/bootstrap.zip"
}

variable "truly_lambda_licenses_function_name" {
  default = "truly_licenses"
  type    = string
}

variable "service_name" {
  default = "licenses"
  type    = string
}

variable "common_tags" { }
variable "resource_logs" {}
variable "resource_dynamodb" {}
variable "resource_xray" {}
variable "resource_secretsman" {}

variable "role" {}


variable "environment_flag" {}

variable "trace_log" {}

variable "lambda_deploy_folder" {}