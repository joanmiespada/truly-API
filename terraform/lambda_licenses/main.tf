locals {
  lambda_file = "${var.lambda_deploy_folder}/${var.lambda_licenses_file}"
}
resource "aws_cloudwatch_log_group" "truly_lambda_licenses_cloudwatch" {
  name              = "/aws/lambda/${var.truly_lambda_licenses_function_name}"
  retention_in_days = 5

  tags = merge(var.common_tags, { service : "${var.service_name}" })
}


resource "aws_lambda_function" "truly_lambda_licenses" {
  function_name    = var.truly_lambda_licenses_function_name
  architectures    = ["arm64"]
  memory_size      = 512
  source_code_hash = filebase64sha256(local.lambda_file)
  filename         = local.lambda_file
  timeout          = 60
  tracing_config {
    mode = "Active"
  }
  handler = "function_handler"
  runtime = "provided.al2"

  role = var.role

  environment {
    variables = {
      ENVIRONMENT      = "${var.environment_flag}"
      RUST_LOG         = "${var.trace_log}"
      BLOCKCHAIN_URL   = "${var.blockchain_url}"
      CONTRACT_ADDRESS = "${var.contract_address}"
      CONTRACT_OWNER   = "${var.contract_owner}"
      KMS_KEY_ID       = "${var.kms_cypher_owner}"
      BLOCKCHAIN_CONFIRMATIONS = "${var.blockchain_confirmations}",
      DEAD_LETTER_QUEUE_MINT= "${var.dead_letter_queue_mint}"
      TOPIC_ARN_MINT_ASYNC = "${var.minting_async_topic_arn}"
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
    var.resoucer_sqs,
    aws_cloudwatch_log_group.truly_lambda_licenses_cloudwatch,
  ]

  tags = merge(var.common_tags, { service : "${var.service_name}" })

}

