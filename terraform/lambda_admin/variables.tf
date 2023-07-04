
variable "lambda_admin_file" {
  description = "The binary zip file for the admin user lambda."
  type    = string
  default = "lambda_admin/bootstrap.zip"
}

variable "truly_lambda_admin_function_name" {
  default = "truly_admin"
  type    = string
}

variable "service_name" {
  default = "admin"
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

variable "rust_backtrace" {
  type= string
}

variable "aws_region" {
  type    = string
}
