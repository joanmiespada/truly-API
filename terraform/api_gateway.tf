
locals {
  route_configurations = [
    aws_apigatewayv2_route.truly_admin_route,
    aws_apigatewayv2_route.truly_licenses_route_asset,
    aws_apigatewayv2_route.truly_licenses_route_asset_by_id,
    aws_apigatewayv2_route.truly_licenses_route_hash_by_id,
    aws_apigatewayv2_route.truly_login_route,
    aws_apigatewayv2_route.truly_user_route,
    aws_apigatewayv2_route.truly_user_route_by_id
    # Add other routes as needed, remember to modify it to force auto-deploy
  ]

  integration_configurations = [
    aws_apigatewayv2_integration.truly_admin_integration,
    aws_apigatewayv2_integration.truly_licenses_integration,
    aws_apigatewayv2_integration.truly_login_integration,
    aws_apigatewayv2_integration.truly_user_integration
    # Add other integrations as needed
  ]

  api_configuration_checksum = sha1(jsonencode({
    routes      = local.route_configurations,
    integrations = local.integration_configurations
  }))
}


resource "aws_apigatewayv2_api" "truly_api" {
  name          = "Truly-API"
  description   = "Truly-API"
  protocol_type = "HTTP"

  cors_configuration {
    allow_origins = ["*"]
    allow_methods = ["POST", "PUT", "GET", "DELETE"]
    allow_headers = ["content-type"]
    max_age       = 300
  }
  tags = merge(local.common_tags, {})
}

resource "aws_apigatewayv2_stage" "truly_stage" {
  api_id      = aws_apigatewayv2_api.truly_api.id
  name        = var.api_stage_version # var.environment_flag  #"$default" "v1"  #
  auto_deploy = false
  tags        = merge(local.common_tags, {})
  deployment_id = aws_apigatewayv2_deployment.truly_api_deployment.id
}

resource "aws_apigatewayv2_stage" "default_stage" { # targe the current one by default
  api_id      = aws_apigatewayv2_api.truly_api.id
  name        = "$default"
  auto_deploy = false
  tags        = merge(local.common_tags, {})
  deployment_id = length(var.active_api_stage) > 0 ? var.active_api_stage : aws_apigatewayv2_deployment.truly_api_deployment.id

  access_log_settings {
    destination_arn = aws_cloudwatch_log_group.api_gateway_logs.arn
    format = "$context.identity.sourceIp - - [$context.requestTime] \"$context.httpMethod $context.routeKey $context.protocol\" $context.status $context.responseLength $context.requestId"
  }

  default_route_settings {
    logging_level = "INFO"
    data_trace_enabled = true  # Enables detailed request/response logging
    throttling_burst_limit = 2000
    throttling_rate_limit  = 5000
  }

}

resource "aws_cloudwatch_log_group" "api_gateway_logs" {
  name = "/aws/apigateway/${aws_apigatewayv2_api.truly_api.name}"
  retention_in_days = 1  # Adjust as needed
}


//---------------- lambda login ----------------------------
resource "aws_apigatewayv2_integration" "truly_login_integration" {
  api_id           = aws_apigatewayv2_api.truly_api.id
  integration_type = "AWS_PROXY"

  connection_type    = "INTERNET"
  description        = "Login methods"
  integration_method = "POST"
  integration_uri    = module.lambda_login.lambda.invoke_arn

  payload_format_version = "2.0"

}

resource "aws_apigatewayv2_route" "truly_login_route" {
  api_id    = aws_apigatewayv2_api.truly_api.id
  route_key = "POST /auth/{proxy+}"
  target    = "integrations/${aws_apigatewayv2_integration.truly_login_integration.id}"

}

resource "aws_lambda_permission" "truly_login_permission" {
  function_name = module.lambda_login.lambda.function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/*${split(" ", aws_apigatewayv2_route.truly_login_route.route_key)[1]}"

}

//---------------- lambda admin ----------------------------
resource "aws_apigatewayv2_integration" "truly_admin_integration" {
  api_id           = aws_apigatewayv2_api.truly_api.id
  integration_type = "AWS_PROXY"

  connection_type    = "INTERNET"
  description        = "Administrative methods, they need authentication and authorization."
  integration_method = "POST"
  integration_uri    = module.lambda_admin.lambda.invoke_arn

  payload_format_version = "2.0"

}

resource "aws_apigatewayv2_route" "truly_admin_route" {
  api_id    = aws_apigatewayv2_api.truly_api.id
  route_key = "ANY /admin/{proxy+}"
  target    = "integrations/${aws_apigatewayv2_integration.truly_admin_integration.id}"

}

resource "aws_lambda_permission" "truly_admin_permission" {
  function_name = module.lambda_admin.lambda.function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_admin_route.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_admin_route.route_key)[1]}"

}

//---------------- lambda my user ----------------------------
resource "aws_apigatewayv2_integration" "truly_user_integration" {
  api_id           = aws_apigatewayv2_api.truly_api.id
  integration_type = "AWS_PROXY"

  connection_type    = "INTERNET"
  description        = "user's methods"
  integration_method = "POST"
  integration_uri    = module.lambda_user.lambda.invoke_arn

  payload_format_version = "2.0"

}

resource "aws_apigatewayv2_route" "truly_user_route" {
  api_id    = aws_apigatewayv2_api.truly_api.id
  route_key = "ANY /api/user"
  target    = "integrations/${aws_apigatewayv2_integration.truly_user_integration.id}"
}
resource "aws_lambda_permission" "truly_user_permission" {
  function_name = module.lambda_user.lambda.function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_user_route.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_user_route.route_key)[1]}"
}
resource "aws_apigatewayv2_route" "truly_user_route_by_id" {
  api_id    = aws_apigatewayv2_api.truly_api.id
  route_key = "ANY /api/user/{id}"
  target    = "integrations/${aws_apigatewayv2_integration.truly_user_integration.id}"
}
resource "aws_lambda_permission" "truly_user_permission_by_id" {
  function_name = module.lambda_user.lambda.function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_user_route_by_id.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_user_route_by_id.route_key)[1]}"
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
  route_key = "ANY /api/asset"
  target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration.id}"

}

resource "aws_lambda_permission" "truly_licenses_permission_asset" {
  function_name = module.lambda_licenses.lambda.function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_asset.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_asset.route_key)[1]}"

}

resource "aws_apigatewayv2_route" "truly_licenses_route_asset_by_id" {
  api_id    = aws_apigatewayv2_api.truly_api.id
  route_key = "ANY /api/asset/{id}"
  target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration.id}"
}

resource "aws_lambda_permission" "truly_licenses_permission_asset_by_id" {
  function_name = module.lambda_licenses.lambda.function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_asset_by_id.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_asset_by_id.route_key)[1]}"
}

resource "aws_apigatewayv2_route" "truly_licenses_route_hash_by_id" {
  api_id    = aws_apigatewayv2_api.truly_api.id
  route_key = "ANY /api/hash/{id}"
  target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration.id}"
}

resource "aws_lambda_permission" "truly_licenses_permission_hash_by_id" {
  function_name = module.lambda_licenses.lambda.function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_hash_by_id.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_hash_by_id.route_key)[1]}"
}

# resource "aws_apigatewayv2_route" "truly_licenses_route_nft" {
#   api_id    = aws_apigatewayv2_api.truly_api.id
#   route_key = "ANY /api/nft"
#   target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration.id}"
# }

# resource "aws_lambda_permission" "truly_licenses_permission_nft" {
#   function_name = module.lambda_licenses.lambda.function_name
#   action        = "lambda:InvokeFunction"
#   principal     = "apigateway.amazonaws.com"
#   source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_nft.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_nft.route_key)[1]}"
# }
# resource "aws_apigatewayv2_route" "truly_licenses_route_license" {
#   api_id    = aws_apigatewayv2_api.truly_api.id
#   route_key = "ANY /api/license"
#   target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration.id}"
# }

# resource "aws_lambda_permission" "truly_licenses_permission_license" {
#   function_name = module.lambda_licenses.lambda.function_name
#   action        = "lambda:InvokeFunction"
#   principal     = "apigateway.amazonaws.com"
#   source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_license.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_license.route_key)[1]}"
# }

# resource "aws_apigatewayv2_route" "truly_licenses_route_license_id" {
#   api_id    = aws_apigatewayv2_api.truly_api.id
#   route_key = "ANY /api/license/{id}"
#   target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration.id}"
# }

# resource "aws_lambda_permission" "truly_licenses_permission_license_id" {
#   function_name = module.lambda_licenses.lambda.function_name
#   action        = "lambda:InvokeFunction"
#   principal     = "apigateway.amazonaws.com"
#   source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_license_id.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_license_id.route_key)[1]}"
# }
# resource "aws_apigatewayv2_route" "truly_licenses_route_asset_by_shorter" {
#   api_id    = aws_apigatewayv2_api.truly_api.id
#   route_key = "ANY /api/shorter"
#   target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration.id}"
# }

# resource "aws_lambda_permission" "truly_licenses_permission_asset_by_shorter" {
#   function_name = module.lambda_licenses.lambda.function_name
#   action        = "lambda:InvokeFunction"
#   principal     = "apigateway.amazonaws.com"
#   source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_asset_by_shorter.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_asset_by_shorter.route_key)[1]}"
# }
# resource "aws_apigatewayv2_route" "truly_licenses_route_asset_by_shorter_id" {
#   api_id    = aws_apigatewayv2_api.truly_api.id
#   route_key = "ANY /api/shorter/{id}"
#   target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration.id}"
# }

# resource "aws_lambda_permission" "truly_licenses_permission_asset_by_shorter_id" {
#   function_name = module.lambda_licenses.lambda.function_name
#   action        = "lambda:InvokeFunction"
#   principal     = "apigateway.amazonaws.com"
#   source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_asset_by_shorter_id.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_asset_by_shorter_id.route_key)[1]}"
# }
# resource "aws_apigatewayv2_route" "truly_licenses_route_tx" {
#   api_id    = aws_apigatewayv2_api.truly_api.id
#   route_key = "GET /api/tx/{id}"
#   target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration.id}"
# }

# resource "aws_lambda_permission" "truly_licenses_permission_tx" {
#   function_name = module.lambda_licenses.lambda.function_name
#   action        = "lambda:InvokeFunction"
#   principal     = "apigateway.amazonaws.com"
#   source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_tx.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_tx.route_key)[1]}"
# }

# resource "aws_apigatewayv2_route" "truly_licenses_route_txs" {
#   api_id    = aws_apigatewayv2_api.truly_api.id
#   route_key = "GET /api/txs/{id}"
#   target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration.id}"
# }

# resource "aws_lambda_permission" "truly_licenses_permission_txs" {
#   function_name = module.lambda_licenses.lambda.function_name
#   action        = "lambda:InvokeFunction"
#   principal     = "apigateway.amazonaws.com"
#   source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_txs.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_txs.route_key)[1]}"
# }
//---------------- register all lambdas below ----------------------------
resource "aws_apigatewayv2_deployment" "truly_api_deployment" {
  api_id      = aws_apigatewayv2_api.truly_api.id
  description = "truly API deployment version: ${var.api_stage_version}"

  lifecycle {
    create_before_destroy = true
  }

  triggers = {
    configuration = local.api_configuration_checksum
  }

  depends_on = [
    aws_apigatewayv2_route.truly_admin_route,
    aws_apigatewayv2_route.truly_licenses_route_asset,
    aws_apigatewayv2_route.truly_licenses_route_asset_by_id,
    aws_apigatewayv2_route.truly_licenses_route_hash_by_id,
    # aws_apigatewayv2_route.truly_licenses_route_asset_by_shorter,
    # aws_apigatewayv2_route.truly_licenses_route_asset_by_shorter_id,
    # aws_apigatewayv2_route.truly_licenses_route_license,
    # aws_apigatewayv2_route.truly_licenses_route_license_id,
    //aws_apigatewayv2_route.truly_licenses_route_nft,
    //aws_apigatewayv2_route.truly_licenses_route_tx,
    //aws_apigatewayv2_route.truly_licenses_route_txs,
    aws_apigatewayv2_route.truly_login_route,
    aws_apigatewayv2_route.truly_user_route,
    aws_apigatewayv2_route.truly_user_route_by_id
  ]

}

