
variable "service_name" {
  type    = string
}

variable "common_tags" { }

variable "role" {}

variable "environment_flag" {}

variable "rust_log" {}


variable "rust_backtrace" {
  type= string
}
variable "aws_region" {
  type    = string
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

variable "email" {
  type = string
  
}

variable "smtp_secret_manager_arn" {
  type = string
  
}
variable "smtp_server" {
  type = string
}

variable "smtp_from" {
  type = string
}