locals {
  region_prefix = element(split("-", var.aws_region), 0)
  lambda_name_descriptor = "${var.common_tags.project}-${var.common_tags.service}-${var.common_tags.environment}-${var.aws_region}-${var.service_name}"
}
resource "aws_cloudwatch_log_group" "truly_lambda_notifications_cloudwatch" {
  name              = "/aws/lambda/${local.lambda_name_descriptor}"
  retention_in_days = 1

  tags = merge(var.common_tags, { "logic" : "${var.service_name}" })
}


resource "aws_lambda_function" "truly_lambda_notifications" {
  function_name = local.lambda_name_descriptor
  architectures = var.architectures
  memory_size   = 512
  timeout       = 90

  package_type = "Image"
  image_uri    = var.ecr_image
  tracing_config {
    mode = "Active"
  }

  role = var.role

  environment {
    variables = {
      ENVIRONMENT         = var.environment_flag
      RUST_LOG            = var.rust_log
      RUST_BACKTRACE      = var.rust_backtrace
      TRACE_LEVEL         = var.trace_level
      SMTP_SECRET_MANAGER = var.smtp_secret_manager_arn
      SMTP_HOST           = var.smtp_server
      SMTP_FROM_EMAIL     = var.smtp_from
      DEFAULT_PAGE_SIZE   = 100
    }
  }

  tags = merge(var.common_tags, { "logic" : "${var.service_name}" })

}

resource "aws_cloudwatch_event_rule" "every_hour" {
  name                = "every-hour"
  description         = "Trigger every hour"
  schedule_expression = "cron(0 * * * ? *)" # every hour
  #schedule_expression = "cron(* * * * ? *)" # every minute, for testing purposes
}

resource "aws_lambda_permission" "allow_cloudwatch" {
  statement_id  = "AllowExecutionFromCloudWatch"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.truly_lambda_notifications.function_name
  principal     = "events.amazonaws.com"
  source_arn    = aws_cloudwatch_event_rule.every_hour.arn
}

resource "aws_cloudwatch_event_target" "every_hour_target" {
  rule      = aws_cloudwatch_event_rule.every_hour.name
  target_id = "LambdaFunction"
  arn       = aws_lambda_function.truly_lambda_notifications.arn
}
