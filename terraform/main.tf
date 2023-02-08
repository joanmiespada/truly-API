terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 3.70.0"
    }
    archive = {
      source  = "hashicorp/archive"
      version = "~> 2.2.0"
    }
  }

  required_version = "~> 1.0"

  #cloud {
    #organization = "mirch"

  #  workspaces {
  #    name = "api"
  #  }
  #}
}

provider "aws" {
  region = var.aws_region
}

data "aws_secretsmanager_secret_version" "secrets" {
  secret_id = var.secrets_key_name 
}

locals {
  #api_secrets = jsondecode(data.aws_secretsmanager_secret_version.secrets.secret_string)

  common_tags = {
    project = var.truly_tag
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

  environment_flag = var.environment_flag
  trace_log = var.trace_log
  lambda_deploy_folder = var.lambda_deploy_folder


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

}

module "lambda_licenses" {
  source = "./lambda_licenses"

  common_tags = local.common_tags 
  resource_logs = aws_iam_role_policy_attachment.truly_lambda_logs
  resource_xray = aws_iam_role_policy_attachment.truly_lambda_XRAY
  resource_dynamodb = aws_iam_role_policy_attachment.truly_lambda_dynamodb
  resource_secretsman = aws_iam_role_policy_attachment.truly_lambda_SECRETSMAN
  resource_kms = aws_iam_role_policy_attachment.truly_lambda_KMS
  role = aws_iam_role.truly_lambda_execution_role.arn

  environment_flag = var.environment_flag
  trace_log = var.trace_log
  lambda_deploy_folder = var.lambda_deploy_folder

}