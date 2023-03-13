terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 4.55.0"
    }
    archive = {
      source  = "hashicorp/archive"
      version = "~> 2.3.0"
    }
  }

  required_version = "~> 1.0"
  
}

provider "aws" {
  region = var.aws_region
}


data "aws_secretsmanager_secret_version" "secrets_app" {
  secret_id = var.secrets_manager_app_keys_name
}
data "aws_secretsmanager_secret_version" "secret_key" {
  secret_id = var.secrets_manager_contract_owner_secret_key_name 
}


locals {

  common_tags = {
    project = var.truly_tag
    service = var.service_tag
  }

}

module "lambda_login" {
  source = "./lambda_login"

  common_tags = local.common_tags 
  resource_logs = aws_iam_role_policy_attachment.truly_lambda_logs
  resource_xray = aws_iam_role_policy_attachment.truly_lambda_XRAY
  resource_dynamodb = aws_iam_role_policy_attachment.truly_lambda_dynamodb
  resource_secretsman = aws_iam_role_policy_attachment.truly_lambda_SECRETSMAN
  role = aws_iam_role.truly_lambda_execution_role.arn
  jwt_token_time_exp_hours = var.jwt_token_time_exp_hours 
  environment_flag = var.environment_flag
  trace_log = var.trace_log
  lambda_deploy_folder = var.lambda_deploy_folder
  rust_backtrace = var.rust_backtrace

}

module "lambda_user" {
  source = "./lambda_user"

  common_tags = local.common_tags 
  resource_logs = aws_iam_role_policy_attachment.truly_lambda_logs
  resource_xray = aws_iam_role_policy_attachment.truly_lambda_XRAY
  resource_dynamodb = aws_iam_role_policy_attachment.truly_lambda_dynamodb
  resource_secretsman = aws_iam_role_policy_attachment.truly_lambda_SECRETSMAN
  role = aws_iam_role.truly_lambda_execution_role.arn

  environment_flag = var.environment_flag
  trace_log = var.trace_log
  lambda_deploy_folder = var.lambda_deploy_folder

  rust_backtrace = var.rust_backtrace
}

module "lambda_admin" {
  source = "./lambda_admin"

  common_tags = local.common_tags 
  resource_logs = aws_iam_role_policy_attachment.truly_lambda_logs
  resource_xray = aws_iam_role_policy_attachment.truly_lambda_XRAY
  resource_dynamodb = aws_iam_role_policy_attachment.truly_lambda_dynamodb
  resource_secretsman = aws_iam_role_policy_attachment.truly_lambda_SECRETSMAN
  role = aws_iam_role.truly_lambda_execution_role.arn

  environment_flag = var.environment_flag
  trace_log = var.trace_log
  lambda_deploy_folder = var.lambda_deploy_folder
  
  rust_backtrace = var.rust_backtrace

}

module "lambda_licenses" {
  source = "./lambda_licenses"

  common_tags = local.common_tags 
  resource_logs = aws_iam_role_policy_attachment.truly_lambda_logs
  resource_xray = aws_iam_role_policy_attachment.truly_lambda_XRAY
  resource_dynamodb = aws_iam_role_policy_attachment.truly_lambda_dynamodb
  resource_secretsman = aws_iam_role_policy_attachment.truly_lambda_SECRETSMAN
  resource_kms = aws_iam_role_policy_attachment.truly_lambda_KMS
  resource_sqs = aws_iam_role_policy_attachment.truly_lambda_SQS
  resource_sns = aws_iam_role_policy_attachment.truly_lambda_SNS
  role = aws_iam_role.truly_lambda_execution_role.arn

  environment_flag = var.environment_flag
  trace_log = var.trace_log
  lambda_deploy_folder = var.lambda_deploy_folder

  blockchain_url = var.blockchain_url
  contract_address = var.contract_address
  contract_owner_address = var.contract_owner_address
  dead_letter_queue_mint = aws_sqs_queue.minting_queue_deadletter.url
  minting_async_topic_arn = aws_sns_topic.minting_topic.arn 
  kms_cypher_owner = var.kms_id_cypher_all_secret_keys

  rust_backtrace = var.rust_backtrace

  video_in_topic = aws_sns_topic.video_in_topic.arn
  video_out_topic = aws_sns_topic.video_out_topic.arn

}
module "lambda_mint" {
  source = "./lambda_mint"

  common_tags = local.common_tags 
  resource_logs = aws_iam_role_policy_attachment.truly_lambda_logs
  resource_xray = aws_iam_role_policy_attachment.truly_lambda_XRAY
  resource_dynamodb = aws_iam_role_policy_attachment.truly_lambda_dynamodb
  resource_secretsman = aws_iam_role_policy_attachment.truly_lambda_SECRETSMAN
  resource_kms = aws_iam_role_policy_attachment.truly_lambda_KMS
  resource_sqs = aws_iam_role_policy_attachment.truly_lambda_SQS
  role = aws_iam_role.truly_lambda_execution_role.arn

  environment_flag = var.environment_flag
  trace_log = var.trace_log
  lambda_deploy_folder = var.lambda_deploy_folder

  blockchain_url = var.blockchain_url
  contract_address = var.contract_address
  contract_owner_address = var.contract_owner_address
  dead_letter_queue_mint = aws_sqs_queue.minting_queue_deadletter.url
  kms_cypher_owner = var.kms_id_cypher_all_secret_keys
  queue_mint_arn = aws_sqs_queue.minting_queue.arn
  
  rust_backtrace = var.rust_backtrace

}