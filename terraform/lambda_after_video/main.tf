locals {
  lambda_file            = "${var.lambda_deploy_folder}/${var.lambda_after_video_file}"
  region_prefix          = element(split("-", var.aws_region), 0)
  lambda_name_descriptor = "${var.truly_lambda_login_function_name}-${local.region_prefix}-${var.api_stage_version}"
}
resource "aws_cloudwatch_log_group" "truly_lambda_after_video_cloudwatch" {
  name              = "/aws/lambda/${local.lambda_name_descriptor}" # ${var.truly_lambda_after_video_function_name}-${local.region_prefix}"
  retention_in_days = 1

  tags = merge(var.common_tags, { service : "${var.service_name}" })
}


resource "aws_lambda_function" "truly_lambda_after_video" {
  function_name    = local.lambda_name_descriptor # var.truly_lambda_after_video_function_name
  architectures    = var.architectures
  memory_size      = 512
  source_code_hash = filebase64sha256(local.lambda_file)
  filename         = local.lambda_file
  timeout          = 60
  tracing_config {
    mode = "Active"
  }
  handler = var.handler # "function_handler"
  runtime = var.runtime # "provided.al2"

  role = var.role

  environment {
    variables = {
      ENVIRONMENT    = "${var.environment_flag}"
      RUST_LOG       = "${var.trace_log}"
      KMS_KEY_ID     = "${var.kms_cypher_owner}"
      RUST_BACKTRACE = "${var.rust_backtrace}"
    }
  }

  depends_on = [
    var.resource_logs,
    var.resource_dynamodb,
    var.resource_xray,
    var.resource_secretsman,
    var.resource_kms,
    var.resource_sqs,
    aws_cloudwatch_log_group.truly_lambda_after_video_cloudwatch,
    var.rust_backtrace
  ]


  tags = merge(var.common_tags, { service : "${var.service_name}" })

}

//--------------- plug queue with lambda after video ----------------

resource "aws_lambda_event_source_mapping" "truly_after_video" {
  event_source_arn = var.sqs_after_video_process_arn
  enabled          = true
  function_name    = aws_lambda_function.truly_lambda_after_video.arn
  batch_size       = 1
}

