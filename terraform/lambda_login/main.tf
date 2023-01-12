locals {
  lambda_file = "${var.lambda_deploy_folder}/${var.lambda_login_file}"
}

resource "aws_cloudwatch_log_group" "truly_lambda_login_cloudwatch" {
  name              = "/aws/lambda/${var.truly_lambda_login_function_name}"
  retention_in_days = 5
  
  tags = merge(var.common_tags,{ service:"${var.service_name}"})
}


resource "aws_lambda_function" "truly_lambda_login" {
  function_name = var.truly_lambda_login_function_name
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

  //role = aws_iam_role.truly_lambda_execution_role.arn
  role = var.role

  environment {
    variables = {
      ENVIRONMENT = "${var.environment_flag}"
      RUST_LOG = "${var.trace_log}" #"cargo_lambda=trace"
    }
  }

  depends_on = [
    var.resource_logs, //aws_iam_role_policy_attachment.truly_lambda_logs,
    var.resource_dynamodb, //aws_iam_role_policy_attachment.truly_lambda_dynamodb,
    //aws_iam_role_policy_attachment.truly_lambda_S3,
    //aws_iam_role_policy_attachment.truly_lambda_SNS,
    var.resource_xray,//aws_iam_role_policy_attachment.truly_lambda_XRAY,
    var.resource_secretsman,
    aws_cloudwatch_log_group.truly_lambda_login_cloudwatch,
  ]
  
  tags = merge(var.common_tags,{ service:"${var.service_name}"})

}

