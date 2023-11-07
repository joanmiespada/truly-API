locals {
  lambda_name_descriptor = "${var.common_tags.project}-${var.common_tags.service}-${var.common_tags.environment}-${var.aws_region}-${var.api_stage_version}-${var.service_name}"
}
resource "aws_cloudwatch_log_group" "truly_lambda_licenses_cloudwatch" {
  name              = "/aws/lambda/${local.lambda_name_descriptor}"
  retention_in_days = 1

  tags = merge(var.common_tags, { "logic" : var.service_name })
}


resource "aws_lambda_function" "truly_lambda_licenses" {
  function_name = local.lambda_name_descriptor
  architectures = var.architectures
  memory_size   = 512
  timeout       = 30
  tracing_config {
    mode = "Active"
  }
  package_type = "Image"
  image_uri    = var.ecr_image

  role = var.role

  environment {
    variables = {
      ENVIRONMENT = var.environment_flag
      RUST_LOG    = var.rust_log
      KMS_KEY_ID  = var.kms_cypher_owner
      RUST_BACKTRACE = var.rust_backtrace
      API_STAGE = var.api_stage_version
      HASHES_SIMILAR_VIDEO_IN_TOPIC = var.hashes_similarities_arn
      MATCHAPI_ENDPOINT             = var.matchapi_endpoint
      TRACE_LEVEL                   = var.trace_level
      URL_BASE_PERMANENT_IMAGES     = var.url_base_permanent_images
      SMTP_HOST                     = var.smtp_server
    }
  }

  depends_on = [
    aws_cloudwatch_log_group.truly_lambda_licenses_cloudwatch,
  ]

  tags = merge(var.common_tags, { "logic" : var.service_name })

}

