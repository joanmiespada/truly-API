
variable "lambda_login_file" {
  description = "The binary zip file for the login lambda."
  type    = string
  default = "../target/lambda/lambda_login/bootstrap.zip"
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

variable "role" {}


variable "environment_flag" {}
variable "aws_region" {}
variable "dynamodb_endpoint" {}