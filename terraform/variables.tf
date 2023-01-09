variable "aws_region" {
  description = "AWS region for all resources."
  type    = string
  default = "eu-central-1"
}

variable "secrets_key_name" {
  description = "secret's manager package."
  type    = string
  default = "truly/api/secrets"
}

variable "dynamodb_endpoint" {
  description = "main dynamodb endpoint"
  type    = string
  default = "https://dynamodb.eu-central-1.amazonaws.com"
}

variable "environment_flag" {
  description = "environment flag"
  type    = string
  default = "production"
}

variable "truly_tag" {
  description = "for tagging all resources linked to this project"
  type = string
  default = "truly"
}

