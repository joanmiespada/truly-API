
variable "lambda_admin_user_file" {
  description = "The binary zip file for the admin user lambda."
  type    = string
  default = "lambda_admin_user/bootstrap.zip"
}

variable "truly_lambda_admin_user_function_name" {
  default = "truly_admin_user"
  type    = string
}

variable "service_name" {
  default = "admin_user"
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