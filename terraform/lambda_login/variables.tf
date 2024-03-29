

variable "service_name" {
  type    = string
}

variable "common_tags" {}

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

variable "trace_level" {
  type=string
}

variable "smtp_server" {
  type = string
}

variable "smtp_from" {
  type = string
}
