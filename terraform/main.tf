terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.5.0"
    }
    archive = {
      source  = "hashicorp/archive"
      version = "~> 2.3.0"
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
  source = "./lambda_login"
  service_name             = "login" 
  common_tags              = local.common_tags
  resource_logs            = aws_iam_role_policy_attachment.truly_lambda_logs
  resource_xray            = aws_iam_role_policy_attachment.truly_lambda_XRAY
  resource_dynamodb        = aws_iam_role_policy_attachment.truly_lambda_dynamodb
  resource_secretsman      = aws_iam_role_policy_attachment.truly_lambda_SECRETSMAN
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

}

module "lambda_user" {
  source = "./lambda_user"

  service_name        = "user" 
  common_tags         = local.common_tags
  resource_logs       = aws_iam_role_policy_attachment.truly_lambda_logs
  resource_xray       = aws_iam_role_policy_attachment.truly_lambda_XRAY
  resource_dynamodb   = aws_iam_role_policy_attachment.truly_lambda_dynamodb
  resource_secretsman = aws_iam_role_policy_attachment.truly_lambda_SECRETSMAN
  role                = aws_iam_role.truly_lambda_execution_role.arn

  environment_flag     = var.environment_flag
  trace_log            = var.trace_log
  rust_log             = var.rust_log

  rust_backtrace    = var.rust_backtrace
  aws_region        = var.aws_region
  api_stage_version = var.api_stage_version
  architectures     = var.architectures
  ecr_image         = var.ecr_user_lambda 
}

module "lambda_admin" {
  source = "./lambda_admin"

  service_name        = "admin" 
  common_tags         = local.common_tags
  resource_logs       = aws_iam_role_policy_attachment.truly_lambda_logs
  resource_xray       = aws_iam_role_policy_attachment.truly_lambda_XRAY
  resource_dynamodb   = aws_iam_role_policy_attachment.truly_lambda_dynamodb
  resource_secretsman = aws_iam_role_policy_attachment.truly_lambda_SECRETSMAN
  role                = aws_iam_role.truly_lambda_execution_role.arn

  environment_flag     = var.environment_flag
  trace_log            = var.trace_log
  rust_log             = var.rust_log

  rust_backtrace    = var.rust_backtrace
  aws_region        = var.aws_region
  api_stage_version = var.api_stage_version
  architectures     = var.architectures
  ecr_image         = var.ecr_admin_lambda 

}

module "lambda_licenses" {
  source = "./lambda_licenses"

  service_name        = "licenses" 
  common_tags         = local.common_tags
  resource_logs       = aws_iam_role_policy_attachment.truly_lambda_logs
  resource_xray       = aws_iam_role_policy_attachment.truly_lambda_XRAY
  resource_dynamodb   = aws_iam_role_policy_attachment.truly_lambda_dynamodb
  resource_secretsman = aws_iam_role_policy_attachment.truly_lambda_SECRETSMAN
  resource_kms        = aws_iam_role_policy_attachment.truly_lambda_KMS
  resource_sqs        = aws_iam_role_policy_attachment.truly_lambda_SQS
  resource_sns        = aws_iam_role_policy_attachment.truly_lambda_SNS
  role                = aws_iam_role.truly_lambda_execution_role.arn

  environment_flag     = var.environment_flag
  trace_log            = var.trace_log
  rust_log             = var.rust_log

  #dead_letter_queue_mint  = aws_sqs_queue.minting_queue_deadletter.url
  #minting_async_topic_arn = aws_sns_topic.minting_topic.arn
  #minting_fails_topic_arn = aws_sns_topic.minting_fails_after_max_retries_topic.arn
  kms_cypher_owner        = var.kms_id_cypher_all_secret_keys

  rust_backtrace = var.rust_backtrace

  #video_in_topic    = aws_sns_topic.video_in_topic.arn
  #video_out_topic   = aws_sns_topic.video_out_topic.arn
  aws_region        = var.aws_region
  api_stage_version = var.api_stage_version
  architectures     = var.architectures
  hashes_similarities_arn = aws_sns_topic.hash_similar_in_topic.arn

  matchapi_endpoint = var.matchapi_endpoint

  ecr_image         = var.ecr_license_lambda 

}
# module "lambda_mint" {
#   source = "./lambda_mint"

#   common_tags         = local.common_tags
#   resource_logs       = aws_iam_role_policy_attachment.truly_lambda_logs
#   resource_xray       = aws_iam_role_policy_attachment.truly_lambda_XRAY
#   resource_dynamodb   = aws_iam_role_policy_attachment.truly_lambda_dynamodb
#   resource_secretsman = aws_iam_role_policy_attachment.truly_lambda_SECRETSMAN
#   resource_kms        = aws_iam_role_policy_attachment.truly_lambda_KMS
#   resource_sqs        = aws_iam_role_policy_attachment.truly_lambda_SQS
#   role                = aws_iam_role.truly_lambda_execution_role.arn

#   environment_flag     = var.environment_flag
#   trace_log            = var.trace_log
#   rust_log             = var.rust_log
#   lambda_deploy_folder = var.lambda_deploy_folder

#   dead_letter_queue_mint  = aws_sqs_queue.minting_queue_deadletter.url
#   kms_cypher_owner        = var.kms_id_cypher_all_secret_keys
#   queue_mint_arn          = aws_sqs_queue.minting_queue.arn
#   minting_async_topic_arn = aws_sns_topic.minting_topic.arn
#   minting_fails_topic_arn = aws_sns_topic.minting_fails_after_max_retries_topic.arn


#   rust_backtrace    = var.rust_backtrace
#   aws_region        = var.aws_region
#   api_stage_version = var.api_stage_version
#   architectures     = var.architectures
#   handler           = var.handler
#   runtime           = var.runtime

# }
# module "lambda_after_video" {
#   source = "./lambda_after_video"

#   common_tags         = local.common_tags
#   resource_logs       = aws_iam_role_policy_attachment.truly_lambda_logs
#   resource_xray       = aws_iam_role_policy_attachment.truly_lambda_XRAY
#   resource_dynamodb   = aws_iam_role_policy_attachment.truly_lambda_dynamodb
#   resource_secretsman = aws_iam_role_policy_attachment.truly_lambda_SECRETSMAN
#   resource_kms        = aws_iam_role_policy_attachment.truly_lambda_KMS
#   resource_sqs        = aws_iam_role_policy_attachment.truly_lambda_SQS
#   role                = aws_iam_role.truly_lambda_execution_role.arn

#   environment_flag     = var.environment_flag
#   trace_log            = var.trace_log
#   rust_log             = var.rust_log
#   lambda_deploy_folder = var.lambda_deploy_folder

#   kms_cypher_owner            = var.kms_id_cypher_all_secret_keys
#   sqs_after_video_process_arn = aws_sqs_queue.after_video_queue.arn

#   rust_backtrace    = var.rust_backtrace
#   aws_region        = var.aws_region
#   api_stage_version = var.api_stage_version
#   architectures     = var.architectures
#   handler           = var.handler
#   runtime           = var.runtime

# }
