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
  api_secrets = jsondecode(data.aws_secretsmanager_secret_version.secrets.secret_string)

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
  role = aws_iam_role.truly_lambda_execution_role.arn

  environment_flag = var.environment_flag
  aws_region = var.aws_region
  dynamodb_endpoint = var.dynamodb_endpoint

}