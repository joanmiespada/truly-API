terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.20.1"
    }
    archive = {
      source  = "hashicorp/archive"
      version = "~> 2.4.0"
    }
  }

  required_version = "~> 1.0"

}

provider "aws" {
  region  = var.aws_region
  profile = "truly"
}

provider "aws" {
  alias   = "default"
  region  = var.aws_region
  profile = "truly"
}

provider "aws" { #only for certificates used by dns
  alias   = "useast"
  region  = "us-east-1"
  profile = "truly"
}

locals {

  region_prefix = element(split("-", var.aws_region), 0)
  common_tags = {
    project     = var.truly_tag
    service     = var.service_tag
    environment = var.environment_flag
  }
}

module "lambda_login" {
  source                   = "./lambda_login"
  service_name             = "login"
  common_tags              = local.common_tags
  # resource_logs            = aws_iam_role_policy_attachment.truly_lambda_logs
  # resource_xray            = aws_iam_role_policy_attachment.truly_lambda_XRAY
  # resource_dynamodb        = aws_iam_role_policy_attachment.truly_lambda_dynamodb
  # resource_secretsman      = aws_iam_role_policy_attachment.truly_lambda_SECRETSMAN
  role                     = aws_iam_role.truly_lambda_execution_role.arn
  jwt_token_time_exp_hours = var.jwt_token_time_exp_hours
  environment_flag         = var.environment_flag
  trace_log                = var.trace_log
  rust_log                 = var.rust_log
  rust_backtrace           = var.rust_backtrace
  aws_region               = var.aws_region
  api_stage_version        = var.api_stage_version
  architectures            = var.architectures
  ecr_image                = var.ecr_login_lambda
  trace_level              = var.trace_level

}

module "lambda_user" {
  source = "./lambda_user"

  service_name        = "user"
  common_tags         = local.common_tags
  # resource_logs       = aws_iam_role_policy_attachment.truly_lambda_logs
  # resource_xray       = aws_iam_role_policy_attachment.truly_lambda_XRAY
  # resource_dynamodb   = aws_iam_role_policy_attachment.truly_lambda_dynamodb
  # resource_secretsman = aws_iam_role_policy_attachment.truly_lambda_SECRETSMAN
  role                = aws_iam_role.truly_lambda_execution_role.arn

  environment_flag = var.environment_flag
  trace_log        = var.trace_log
  rust_log         = var.rust_log

  rust_backtrace    = var.rust_backtrace
  aws_region        = var.aws_region
  api_stage_version = var.api_stage_version
  architectures     = var.architectures
  ecr_image         = var.ecr_user_lambda
  trace_level       = var.trace_level
}

module "lambda_admin" {
  source = "./lambda_admin"

  service_name        = "admin"
  common_tags         = local.common_tags
  # resource_logs       = aws_iam_role_policy_attachment.truly_lambda_logs
  # resource_xray       = aws_iam_role_policy_attachment.truly_lambda_XRAY
  # resource_dynamodb   = aws_iam_role_policy_attachment.truly_lambda_dynamodb
  # resource_secretsman = aws_iam_role_policy_attachment.truly_lambda_SECRETSMAN
  role                = aws_iam_role.truly_lambda_execution_role.arn

  environment_flag = var.environment_flag
  trace_log        = var.trace_log
  rust_log         = var.rust_log

  rust_backtrace    = var.rust_backtrace
  aws_region        = var.aws_region
  api_stage_version = var.api_stage_version
  architectures     = var.architectures
  ecr_image         = var.ecr_admin_lambda
  trace_level       = var.trace_level
}

module "lambda_licenses" {
  source = "./lambda_licenses"

  service_name     = "licenses"
  common_tags      = local.common_tags
  role             = aws_iam_role.truly_lambda_execution_role.arn
  environment_flag = var.environment_flag
  rust_log         = var.rust_log
  kms_cypher_owner = var.kms_id_cypher_all_secret_keys

  rust_backtrace = var.rust_backtrace

  aws_region              = var.aws_region
  api_stage_version       = var.api_stage_version
  architectures           = var.architectures
  hashes_similarities_arn = aws_sns_topic.video_in_topic.arn 

  matchapi_endpoint = var.matchapi_endpoint

  ecr_image = var.ecr_license_lambda

  trace_level = var.trace_level

  url_base_permanent_images = "https://cdn.${var.dns_prefix}.${var.dns_base}"

}

module "lambda_after_hash" {
  source = "./lambda_after_hash"

  service_name     = "after_hash"
  common_tags      = local.common_tags
  role             = aws_iam_role.truly_lambda_execution_role.arn
  environment_flag = var.environment_flag
  rust_log         = var.rust_log

  rust_backtrace = var.rust_backtrace

  aws_region    = var.aws_region
  architectures = var.architectures

  ecr_image = var.ecr_after_hash_lambda

  trace_level = var.trace_level

  email               = var.email
  video_out_topic_arn = aws_sns_topic.video_out_topic.arn

}

module "lambda_error" {
  source = "./lambda_error"

  service_name     = "error"
  common_tags      = local.common_tags
  role             = aws_iam_role.truly_lambda_execution_role.arn
  environment_flag = var.environment_flag
  rust_log         = var.rust_log

  rust_backtrace = var.rust_backtrace

  aws_region    = var.aws_region
  architectures = var.architectures

  ecr_image = var.ecr_error_lambda

  trace_level = var.trace_level

  email = var.email

  video_error_topic_arn = aws_sns_topic.video_error_topic.arn

}

module "lambda_alert_similar" {
  source = "./lambda_alert_similar"

  service_name     = "alert_similar"
  common_tags      = local.common_tags
  role             = aws_iam_role.truly_lambda_execution_role.arn
  environment_flag = var.environment_flag
  rust_log         = var.rust_log

  rust_backtrace = var.rust_backtrace

  aws_region    = var.aws_region
  architectures = var.architectures

  ecr_image = var.ecr_alert_similars_lambda

  trace_level = var.trace_level

  email                   = var.email
  alert_similar_topic_arn = aws_sns_topic.notify_new_similar_topic.arn

}

module "lambda_notifications" {
  source = "./lambda_notifications"

  service_name     = "notifications"
  common_tags      = local.common_tags
  role             = aws_iam_role.truly_lambda_execution_role.arn
  environment_flag = var.environment_flag
  rust_log         = var.rust_log

  rust_backtrace = var.rust_backtrace

  aws_region    = var.aws_region
  architectures = var.architectures

  ecr_image = var.ecr_notifications_lambda

  trace_level = var.trace_level

  email                   = var.email

  smtp_secret = aws_secretsmanager_secret.smtp_secret.arn
  smtp_server = aws_ses_domain_identity.email_ses_sender_domain.domain

}