locals {
  region_prefix          = element(split("-", var.aws_region), 0)
  lambda_name_descriptor = "${var.common_tags.project}-${var.common_tags.service}-${var.common_tags.environment}-${var.aws_region}-${var.api_stage_version}-${var.service_name}"
}

resource "aws_cloudwatch_log_group" "truly_lambda_admin_cloudwatch" {
  name              = "/aws/lambda/${local.lambda_name_descriptor}" 
  retention_in_days = 1

  tags = merge(var.common_tags, { logic : "${var.service_name}" })
}


resource "aws_lambda_function" "truly_lambda_admin" {
  function_name = local.lambda_name_descriptor 
  architectures = var.architectures
  memory_size   = 512
  timeout       = 30
  package_type  = "Image"
  image_uri     = var.ecr_image
  tracing_config {
    mode = "Active"
  }

  role = var.role

  environment {
    variables = {
      ENVIRONMENT    = var.environment_flag
      RUST_LOG       = var.rust_log
      RUST_BACKTRACE = var.rust_backtrace
      API_STAGE      = var.api_stage_version
      TRACE_LEVEL    = var.trace_level
      SMTP_HOST       = var.smtp_server
      SMTP_FROM_EMAIL = var.smtp_from
    }
  }

  depends_on = [
    aws_cloudwatch_log_group.truly_lambda_admin_cloudwatch,
  ]

  tags = merge(var.common_tags, { "logic" : "${var.service_name}" })

}

