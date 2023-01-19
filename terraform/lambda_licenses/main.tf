locals {
  lambda_file = "${var.lambda_deploy_folder}/${var.lambda_licenses_file}"
}
resource "aws_cloudwatch_log_group" "truly_lambda_licenses_cloudwatch" {
  name              = "/aws/lambda/${var.truly_lambda_licenses_function_name}"
  retention_in_days = 5
  
  tags = merge(var.common_tags,{ service:"${var.service_name}"})
}


resource "aws_lambda_function" "truly_lambda_licenses" {
  function_name = var.truly_lambda_licenses_function_name
  architectures = [ "arm64" ]
  memory_size = 512
  source_code_hash = filebase64sha256(local.lambda_file)
  filename         =  local.lambda_file 
  timeout = 60
  tracing_config {
    mode="Active"
  }
  handler = "function_handler"
  runtime = "provided.al2"

  role = var.role

  environment {
    variables = {
      ENVIRONMENT = "${var.environment_flag}"
      RUST_LOG = "${var.trace_log}"
    }
  }

  depends_on = [
    var.resource_logs, 
    var.resource_dynamodb, 
    //aws_iam_role_policy_attachment.truly_lambda_S3,
    //aws_iam_role_policy_attachment.truly_lambda_SNS,
    var.resource_xray,
    var.resource_secretsman,
    aws_cloudwatch_log_group.truly_lambda_licenses_cloudwatch,
  ]
  
  tags = merge(var.common_tags,{ service:"${var.service_name}"})

}

