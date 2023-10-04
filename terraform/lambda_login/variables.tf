

variable "service_name" {
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
variable "rust_log" {}

variable "jwt_token_time_exp_hours" {
  description = "time expiration jwt in production based on hours"
  type        = number
}

variable "rust_backtrace" {
  type = string
}
variable "aws_region" {
  type = string
}
variable "api_stage_version" {
  type = string
}
variable "architectures" {
  type    = list(string)
}

variable "ecr_image" {
  type = string
  description = "ecr repo where I must pull the image base"
}


