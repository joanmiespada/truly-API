
variable "lambda_login_file" {
  description = "The binary zip file for the login lambda."
  type    = string
  default = "lambda_login/bootstrap.zip"
}

variable "truly_lambda_login_function_name" {
  default = "truly_login"
  type    = string
}

variable "service_name" {
  default = "login"
  type    = string
}

variable "common_tags" { }
variable "resource_logs" {}
variable "resource_dynamodb" {}
variable "resource_xray" {}
variable "resource_secretsman" {}

variable "role" {}
variable "regions" {}


variable "environment_flag" {}

variable "trace_log" {}

variable "lambda_deploy_folder" {}
variable "jwt_token_time_exp_hours" {
  description = "time expiration jwt in production based on hours"
  type    = number
}

variable "rust_backtrace" {
  type= string
}

variable "function_handler" {}
variable "runtime" {}
variable "architecture" {}