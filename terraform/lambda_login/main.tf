locals {
  lambda_file            = "${var.lambda_deploy_folder}/${var.lambda_login_file}"
  region_prefix          = element(split("-", var.aws_region), 0)
  #lambda_name_descriptor = "${var.truly_lambda_login_function_name}-${local.region_prefix}-${var.api_stage_version}"
  lambda_name_descriptor = "${var.common_tags.project}-${var.common_tags.service}-${var.common_tags.environment}-${var.aws_region}-${var.api_stage_version}-${var.service_name}"
}

resource "aws_cloudwatch_log_group" "truly_lambda_login_cloudwatch" {
  name              = "/aws/lambda/${local.lambda_name_descriptor}" #"/aws/lambda/${var.truly_lambda_login_function_name}-${local.region_prefix}"
  retention_in_days = 1

  tags = merge(var.common_tags, { logic : "${var.service_name}" })
}


resource "aws_lambda_function" "truly_lambda_login" {
  function_name    = local.lambda_name_descriptor #var.truly_lambda_login_function_name
  architectures    = var.architectures            # [ "arm64" ]
  memory_size      = 512
  source_code_hash = filebase64sha256(local.lambda_file)
  filename         = local.lambda_file
  timeout          = 60
  tracing_config {
    mode = "Active"
  }
  handler = var.handler # "function_handler"
  runtime = var.runtime # "provided.al2"

  //role = aws_iam_role.truly_lambda_execution_role.arn
  role = var.role

  environment {
    variables = {
      ENVIRONMENT              = "${var.environment_flag}"
      RUST_LOG                 = "${var.rust_log}"
      JWT_TOKEN_TIME_EXP_HOURS = "${var.jwt_token_time_exp_hours}"
      RUST_BACKTRACE           = "${var.rust_backtrace}"
      API_STAGE                = "${var.api_stage_version}"
    }
  }

  depends_on = [
    var.resource_logs,     //aws_iam_role_policy_attachment.truly_lambda_logs,
    var.resource_dynamodb, //aws_iam_role_policy_attachment.truly_lambda_dynamodb,
    //aws_iam_role_policy_attachment.truly_lambda_S3,
    //aws_iam_role_policy_attachment.truly_lambda_SNS,
    var.resource_xray, //aws_iam_role_policy_attachment.truly_lambda_XRAY,
    var.resource_secretsman,
    aws_cloudwatch_log_group.truly_lambda_login_cloudwatch,
    var.rust_backtrace
  ]

  tags = merge(var.common_tags, { service : "${var.service_name}" })

}

