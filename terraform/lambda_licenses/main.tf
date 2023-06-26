locals {
  lambda_file = "${var.lambda_deploy_folder}/${var.lambda_licenses_file}"
}
resource "aws_cloudwatch_log_group" "truly_lambda_licenses_cloudwatch" {
  for_each = toset(var.regions)
  region   = each.key

  name              = "/aws/lambda/${var.truly_lambda_licenses_function_name}"
  retention_in_days = 2

  tags = merge(var.common_tags, { service : "${var.service_name}" })
}


resource "aws_lambda_function" "truly_lambda_licenses" {
  for_each = toset(var.regions)
  region   = each.key

  function_name    = var.truly_lambda_licenses_function_name
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
      DEAD_LETTER_QUEUE_MINT= var.dead_letter_queue_mint[each.key]
      TOPIC_ARN_MINT_ASYNC = var.minting_async_topic_arn[each.key].arn
      RUST_BACKTRACE = "${var.rust_backtrace}"
      SHORTER_VIDEO_IN_TOPIC = var.video_in_topic[each.key]
      SHORTER_VIDEO_OUT_TOPIC = var.video_out_topic[each.key]
      MINTING_FAILS_TOPIC =  var.minting_fails_topic_arn[each.key].arn 
    }
  }

  depends_on = [
    var.resource_logs,
    var.resource_dynamodb,
    //aws_iam_role_policy_attachment.truly_lambda_S3,
    //aws_iam_role_policy_attachment.truly_lambda_SNS,
    var.resource_xray,
    var.resource_secretsman,
    var.resource_kms,
    var.resource_sqs,
    var.resource_sns,
    aws_cloudwatch_log_group.truly_lambda_licenses_cloudwatch,
    var.dead_letter_queue_mint,
    var.minting_async_topic_arn,
    var.rust_backtrace
  ]

  tags = merge(var.common_tags, { service : "${var.service_name}" })

}

