locals {
  lambda_file = "${var.lambda_deploy_folder}/${var.lambda_after_video_file}"
}
resource "aws_cloudwatch_log_group" "truly_lambda_after_video_cloudwatch" {
  for_each = toset(var.regions)
  region   = each.key

  name              = "/aws/lambda/${var.truly_lambda_after_video_function_name}"
  retention_in_days = 2

  tags = merge(var.common_tags, { service : "${var.service_name}" })
}


resource "aws_lambda_function" "truly_lambda_after_video" {
  for_each = toset(var.regions)
  region   = each.key

  function_name    = var.truly_lambda_after_video_function_name
  architectures    = var.architecture
  memory_size      = 512
  source_code_hash = filebase64sha256(local.lambda_file)
  filename         = local.lambda_file
  timeout          = 60
  tracing_config {
    mode = "Active"
  }
  handler = var.function_handler
  runtime = var.runtime

  role = var.role

  environment {
    variables = {
      ENVIRONMENT      = "${var.environment_flag}"
      RUST_LOG         = "${var.trace_log}"
      KMS_KEY_ID       = "${var.kms_cypher_owner}"
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
  for_each = toset(var.regions)
  region   = each.key

  event_source_arn = var.sqs_after_video_process_arn[each.key].arn
  enabled = true
  function_name    = aws_lambda_function.truly_lambda_after_video[each.key].arn
  batch_size = 1 // only 1 message per lambda
}

