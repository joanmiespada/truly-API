resource "aws_apigatewayv2_api" "truly_api" {
  name          = "Truly API"
  description   = "Truly API"
  protocol_type = "HTTP"

  cors_configuration {
    allow_origins = ["*"]
    allow_methods = ["POST", "PUT", "GET", "DELETE"]
    allow_headers = ["content-type"]
    max_age = 300
  }
  tags = merge(local.common_tags,{})
}

resource "aws_apigatewayv2_stage" "truly_stage" {
  api_id      = aws_apigatewayv2_api.truly_api.id
  name        = "$default"
  auto_deploy = true
  tags = merge(local.common_tags,{})
}

resource "aws_apigatewayv2_integration" "truly_login_integration" {
  api_id           = aws_apigatewayv2_api.truly_api.id
  integration_type = "AWS_PROXY"

  connection_type    = "INTERNET"
  description        = "Login methods"
  integration_method = "POST"
  integration_uri    = module.lambda_login.lambda.invoke_arn  #lambda_login.aws_lambda_function.truly_lambda_login.invoke_arn

  payload_format_version = "2.0"

}

resource "aws_apigatewayv2_route" "truly_login_route" {
  api_id    = aws_apigatewayv2_api.truly_api.id
  route_key = "POST /auth/login"
  target    = "integrations/${aws_apigatewayv2_integration.truly_login_integration.id}"

}

resource "aws_lambda_permission" "truly_login_permission" {
  function_name = module.lambda_login.lambda.function_name   
  #aws_lambda_function.truly_lambda_login.function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/*/${split("/", aws_apigatewayv2_route.truly_login_route.route_key)[1]}"

}

resource "aws_apigatewayv2_deployment" "truly_api_deployment" {
  api_id      = aws_apigatewayv2_api.truly_api.id
  description = "truly API deployment"

  triggers = {
    redeployment = sha1(join(",", [
      jsonencode(aws_apigatewayv2_integration.truly_login_integration),
      jsonencode(aws_apigatewayv2_route.truly_login_route),
      ],
    ))
  }

  lifecycle {
    create_before_destroy = true
  }
}

