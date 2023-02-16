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

//---------------- lambda login ----------------------------
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
  route_key =  "POST /auth/{proxy+}"
  target    = "integrations/${aws_apigatewayv2_integration.truly_login_integration.id}"

}

resource "aws_lambda_permission" "truly_login_permission" {
  function_name = module.lambda_login.lambda.function_name   
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_login_route.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_login_route.route_key)[1]}"
  //source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/POST/auth/login"

}

//---------------- lambda admin ----------------------------
resource "aws_apigatewayv2_integration" "truly_admin_integration" {
  api_id           = aws_apigatewayv2_api.truly_api.id
  integration_type = "AWS_PROXY"

  connection_type    = "INTERNET"
  description        = "Login methods"
  integration_method = "POST"
  integration_uri    = module.lambda_admin.lambda.invoke_arn

  payload_format_version = "2.0"

}

resource "aws_apigatewayv2_route" "truly_admin_route" {
  api_id    = aws_apigatewayv2_api.truly_api.id
  route_key =  "ANY /admin/{proxy+}"
  target    = "integrations/${aws_apigatewayv2_integration.truly_admin_integration.id}"

}

resource "aws_lambda_permission" "truly_admin_permission" {
  function_name = module.lambda_admin.lambda.function_name   
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_admin_route.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_admin_route.route_key)[1]}"
  //source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/POST/auth/login"

}

//---------------- lambda my user ----------------------------
resource "aws_apigatewayv2_integration" "truly_user_integration" {
  api_id           = aws_apigatewayv2_api.truly_api.id
  integration_type = "AWS_PROXY"

  connection_type    = "INTERNET"
  description        = "Login methods"
  integration_method = "POST"
  integration_uri    = module.lambda_user.lambda.invoke_arn

  payload_format_version = "2.0"

}

resource "aws_apigatewayv2_route" "truly_user_route" {
  api_id    = aws_apigatewayv2_api.truly_api.id
  route_key =  "ANY /api/user/{proxy+}"
  target    = "integrations/${aws_apigatewayv2_integration.truly_user_integration.id}"

}

resource "aws_lambda_permission" "truly_user_permission" {
  function_name = module.lambda_user.lambda.function_name   
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_user_route.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_user_route.route_key)[1]}"
  //source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/POST/auth/login"

}

//---------------- lambda license ----------------------------
resource "aws_apigatewayv2_integration" "truly_licenses_integration" {
  api_id           = aws_apigatewayv2_api.truly_api.id
  integration_type = "AWS_PROXY"

  connection_type    = "INTERNET"
  description        = "Licenses, assets and ownership methods"
  integration_method = "POST"
  integration_uri    = module.lambda_licenses.lambda.invoke_arn

  payload_format_version = "2.0"

}

resource "aws_apigatewayv2_route" "truly_licenses_route_asset" {
  api_id    = aws_apigatewayv2_api.truly_api.id
  route_key =  "ANY /api/asset/{proxy+}"
  target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration.id}"

}

resource "aws_lambda_permission" "truly_licenses_permission_asset" {
  function_name = module.lambda_licenses.lambda.function_name   
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_asset.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_asset.route_key)[1]}"
  //source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/POST/auth/login"

}
resource "aws_apigatewayv2_route" "truly_licenses_route_owner" {
  api_id    = aws_apigatewayv2_api.truly_api.id
  route_key =  "ANY /api/owner/{proxy+}"
  target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration.id}"

}

resource "aws_lambda_permission" "truly_licenses_permission_owner" {
  function_name = module.lambda_licenses.lambda.function_name   
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_owner.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_owner.route_key)[1]}"
  //source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/POST/auth/login"

}

resource "aws_apigatewayv2_route" "truly_licenses_route_nft" {
  api_id    = aws_apigatewayv2_api.truly_api.id
  route_key =  "ANY /api/nft/{proxy+}"
  target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration.id}"

}

resource "aws_lambda_permission" "truly_licenses_permission_nft" {
  function_name = module.lambda_licenses.lambda.function_name   
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_nft.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_nft.route_key)[1]}"
  //source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/POST/auth/login"

}

//---------------- register all lambdas below ----------------------------
resource "aws_apigatewayv2_deployment" "truly_api_deployment" {
  api_id      = aws_apigatewayv2_api.truly_api.id
  description = "truly API deployment"

  triggers = {
    redeployment = sha1(join(",", [
      jsonencode(aws_apigatewayv2_integration.truly_login_integration),
      jsonencode(aws_apigatewayv2_route.truly_login_route),

      jsonencode(aws_apigatewayv2_integration.truly_admin_integration),
      jsonencode(aws_apigatewayv2_route.truly_admin_route),

      jsonencode(aws_apigatewayv2_integration.truly_user_integration),
      jsonencode(aws_apigatewayv2_route.truly_user_route),

      jsonencode(aws_apigatewayv2_integration.truly_licenses_integration),
      jsonencode(aws_apigatewayv2_route.truly_licenses_route_owner),
      jsonencode(aws_apigatewayv2_route.truly_licenses_route_asset),
      jsonencode(aws_apigatewayv2_route.truly_licenses_route_nft),
      ],
    ))
  }

  lifecycle {
    create_before_destroy = true
  }
}

