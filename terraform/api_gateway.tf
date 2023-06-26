resource "aws_apigatewayv2_api" "truly_api" {
  for_each = toset(var.regions)
  region   = each.key

  name          = "Truly API"
  description   = "Truly API"
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
  for_each = toset(var.regions)
  region   = each.key

  api_id      =  aws_apigatewayv2_api.truly_api[each.key].id
  name        =  var.environment_flag  //"$default"
  auto_deploy = true
  tags        = merge(local.common_tags, {})
}
//---------------- lambda login ----------------------------
resource "aws_apigatewayv2_integration" "truly_login_integration" {
  for_each = toset(var.regions)
  region   = each.key

  api_id           = aws_apigatewayv2_api.truly_api[each.key].id
  integration_type = "AWS_PROXY"

  connection_type    = "INTERNET"
  description        = "Login methods"
  integration_method = "POST"
  integration_uri    = module.lambda_login.lambdas[each.key].invoke_arn  

  payload_format_version = "2.0"

}

resource "aws_apigatewayv2_route" "truly_login_route" {
  for_each = toset(var.regions)
  region   = each.key

  api_id      = aws_apigatewayv2_api.truly_api[each.key].id
  route_key = "POST /auth/{proxy+}"
  target    = "integrations/${aws_apigatewayv2_integration.truly_login_integration[each.key].id}"

}

resource "aws_lambda_permission" "truly_login_permission" {
  for_each = toset(var.regions)
  region   = each.key

  function_name = module.lambda_login.lambdas[each.key].function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api[each.key].execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_login_route[each.key].route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_login_route.route_key[each.key])[1]}"

}


//---------------- lambda admin ----------------------------
resource "aws_apigatewayv2_integration" "truly_admin_integration" {
  for_each = toset(var.regions)
  region   = each.key

  api_id           = aws_apigatewayv2_api.truly_api[each.key].id
  integration_type = "AWS_PROXY"

  connection_type    = "INTERNET"
  description        = "Login methods"
  integration_method = "POST"
  integration_uri    = module.lambda_admin.lambdas[each.key].invoke_arn

  payload_format_version = "2.0"

}

resource "aws_apigatewayv2_route" "truly_admin_route" {
  for_each = toset(var.regions)
  region   = each.key

  api_id    = aws_apigatewayv2_api.truly_api[each.key].id
  route_key = "ANY /admin/{proxy+}"
  target    = "integrations/${aws_apigatewayv2_integration.truly_admin_integration[each.key].id}"

}

resource "aws_lambda_permission" "truly_admin_permission" {
  for_each = toset(var.regions)
  region   = each.key

  function_name = module.lambda_admin.lambdas[each.key].function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api[each.key].execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_admin_route[each.key].route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_admin_route.route_key[each.key])[1]}"

}

//---------------- lambda my user ----------------------------
resource "aws_apigatewayv2_integration" "truly_user_integration" {
  for_each = toset(var.regions)
  region   = each.key

  //api_id           = aws_apigatewayv2_api.truly_api.id
  api_id           = aws_apigatewayv2_api.truly_api[each.key].id
  integration_type = "AWS_PROXY"

  connection_type    = "INTERNET"
  description        = "user's methods"
  integration_method = "POST"
  //integration_uri    = module.lambda_user.lambda.invoke_arn
  integration_uri    = module.lambda_user.lambdas[each.key].invoke_arn

  payload_format_version = "2.0"

}
resource "aws_apigatewayv2_route" "truly_user_route" {
  for_each = toset(var.regions)
  region   = each.key

  api_id    = aws_apigatewayv2_api.truly_api[each.key].id
  route_key = "ANY /api/user"
  target    = "integrations/${aws_apigatewayv2_integration.truly_user_integration[each.key].id}"
}
resource "aws_lambda_permission" "truly_user_permission" {
  for_each = toset(var.regions)
  region   = each.key

  function_name = module.lambda_user.lambdas[each.key].function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  //source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_user_route.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_user_route.route_key)[1]}"
  source_arn    = "${aws_apigatewayv2_api.truly_api[each.key].execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_user_route[each.key].route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_user_route[each.key].route_key)[1]}"
}
resource "aws_apigatewayv2_route" "truly_user_route_by_id" {
  for_each = toset(var.regions)
  region   = each.key

  //api_id    = aws_apigatewayv2_api.truly_api.id
  api_id    = aws_apigatewayv2_api.truly_api[each.key].id
  route_key = "ANY /api/user/{id}"
  target    = "integrations/${aws_apigatewayv2_integration.truly_user_integration[each.key].id}"
}
resource "aws_lambda_permission" "truly_user_permission_by_id" {
  for_each = toset(var.regions)
  region   = each.key

  function_name = module.lambda_user.lambdas[each.key].function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  //source_arn    = "${aws_apigatewayv2_api.truly_api.execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_user_route_by_id.route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_user_route_by_id.route_key)[1]}"
  source_arn    = "${aws_apigatewayv2_api.truly_api[each.key].execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_user_route_by_id[each.key].route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_user_route_by_id[each.key].route_key)[1]}"
}

//---------------- lambda license ----------------------------
resource "aws_apigatewayv2_integration" "truly_licenses_integration" {
  for_each = toset(var.regions)
  region   = each.key

  //api_id           = aws_apigatewayv2_api.truly_api.id
  api_id           = aws_apigatewayv2_api.truly_api[each.key].id
  integration_type = "AWS_PROXY"

  connection_type    = "INTERNET"
  description        = "Licenses, assets and ownership methods"
  integration_method = "POST"
  //integration_uri    = module.lambda_licenses.lambda.invoke_arn
  integration_uri    = module.lambda_licenses.lambdas[each.key].invoke_arn

  payload_format_version = "2.0"

}

resource "aws_apigatewayv2_route" "truly_licenses_route_asset" {
  for_each = toset(var.regions)
  region   = each.key

  //api_id    = aws_apigatewayv2_api.truly_api.id
  api_id    = aws_apigatewayv2_api.truly_api[each.key].id
  route_key = "ANY /api/asset"
  target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration[each.key].id}"

}

resource "aws_lambda_permission" "truly_licenses_permission_asset" {
  for_each = toset(var.regions)
  region   = each.key

  function_name = module.lambda_licenses.lambdas[each.key].function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api[each.key].execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_asset[each.key].route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_asset[each.key].route_key)[1]}"
}

resource "aws_apigatewayv2_route" "truly_licenses_route_asset_by_id" {
  for_each = toset(var.regions)
  region   = each.key

  //api_id    = aws_apigatewayv2_api.truly_api.id
  api_id    = aws_apigatewayv2_api.truly_api[each.key].id
  route_key = "ANY /api/asset/{id}"
  target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration[each.key].id}"
}

resource "aws_lambda_permission" "truly_licenses_permission_asset_by_id" {
  for_each = toset(var.regions)
  region   = each.key

  function_name = module.lambda_licenses.lambdas[each.key].function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api[each.key].execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_asset_by_id[each.key].route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_asset_by_id[each.key].route_key)[1]}"
}


resource "aws_apigatewayv2_route" "truly_licenses_route_nft" {
  for_each = toset(var.regions)
  region   = each.key

  //api_id    = aws_apigatewayv2_api.truly_api.id
  api_id    = aws_apigatewayv2_api.truly_api[each.key].id
  route_key = "ANY /api/nft"
  target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration[each.key].id}"
}

resource "aws_lambda_permission" "truly_licenses_permission_nft" {
  for_each = toset(var.regions)
  region   = each.key

  function_name = module.lambda_licenses.lambdas[each.key].function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api[each.key].execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_nft[each.key].route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_nft[each.key].route_key)[1]}"
}
resource "aws_apigatewayv2_route" "truly_licenses_route_license" {
  for_each = toset(var.regions)
  region   = each.key

  //api_id    = aws_apigatewayv2_api.truly_api.id
  api_id    = aws_apigatewayv2_api.truly_api[each.key].id
  route_key = "ANY /api/license"
  target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration[each.key].id}"
}

resource "aws_lambda_permission" "truly_licenses_permission_license" {
  for_each = toset(var.regions)
  region   = each.key

  function_name = module.lambda_licenses.lambdas[each.key].function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api[each.key].execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_license[each_key].route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_license[each_key].route_key)[1]}"
}

resource "aws_apigatewayv2_route" "truly_licenses_route_license_id" {
  for_each = toset(var.regions)
  region   = each.key

  //api_id    = aws_apigatewayv2_api.truly_api.id
  api_id    = aws_apigatewayv2_api.truly_api[each.key].id
  route_key = "ANY /api/license/{id}"
  target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration[each.key].id}"
}

resource "aws_lambda_permission" "truly_licenses_permission_license_id" {
  for_each = toset(var.regions)
  region   = each.key

  function_name = module.lambda_licenses.lambdas[each.key].function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api[each.key].execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_license_id[each.key].route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_license_id[each.key].route_key)[1]}"
}
resource "aws_apigatewayv2_route" "truly_licenses_route_asset_by_shorter" {
  for_each = toset(var.regions)
  region   = each.key

  //api_id    = aws_apigatewayv2_api.truly_api.id
  api_id    = aws_apigatewayv2_api.truly_api[each.key].id
  route_key = "ANY /api/shorter"
  target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration[each.key].id}"
}

resource "aws_lambda_permission" "truly_licenses_permission_asset_by_shorter" {
  for_each = toset(var.regions)
  region   = each.key

  function_name = module.lambda_licenses.lambdas[each.key].function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api[each.key].execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_asset_by_shorter[each.key].route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_asset_by_shorter[each.key].route_key)[1]}"
}
resource "aws_apigatewayv2_route" "truly_licenses_route_asset_by_shorter_id" {
  for_each = toset(var.regions)
  region   = each.key

  //api_id    = aws_apigatewayv2_api.truly_api.id
  api_id    = aws_apigatewayv2_api.truly_api[each.key].id
  route_key = "ANY /api/shorter/{id}"
  target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration[each.key].id}"
}

resource "aws_lambda_permission" "truly_licenses_permission_asset_by_shorter_id" {
  for_each = toset(var.regions)
  region   = each.key

  function_name = module.lambda_licenses.lambdas[each.key].function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api[each.key].execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_asset_by_shorter_id[each.key].route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_asset_by_shorter_id[each.key].route_key)[1]}"
}
resource "aws_apigatewayv2_route" "truly_licenses_route_tx" {
  for_each = toset(var.regions)
  region   = each.key

  //api_id    = aws_apigatewayv2_api.truly_api.id
  api_id    = aws_apigatewayv2_api.truly_api[each.key].id
  route_key = "GET /api/tx/{id}"
  target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration[each.key].id}"
}

resource "aws_lambda_permission" "truly_licenses_permission_tx" {
  for_each = toset(var.regions)
  region   = each.key

  function_name = module.lambda_licenses.lambdas[each.key].function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api[each.key].execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_tx[each.key].route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_tx[each.key].route_key)[1]}"
}

resource "aws_apigatewayv2_route" "truly_licenses_route_txs" {
  for_each = toset(var.regions)
  region   = each.key

  //api_id    = aws_apigatewayv2_api.truly_api.id
  api_id    = aws_apigatewayv2_api.truly_api[each.key].id
  route_key = "GET /api/txs/{id}"
  target    = "integrations/${aws_apigatewayv2_integration.truly_licenses_integration[each.key].id}"
}

resource "aws_lambda_permission" "truly_licenses_permission_txs" {
  for_each = toset(var.regions)
  region   = each.key

  function_name = module.lambda_licenses.lambdas[each.key].function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.truly_api[each.key].execution_arn}/*/${split(" ", aws_apigatewayv2_route.truly_licenses_route_txs[each.key].route_key)[0]}${split(" ", aws_apigatewayv2_route.truly_licenses_route_txs[each.key].route_key)[1]}"
}
//---------------- register all lambdas below ----------------------------
resource "aws_apigatewayv2_deployment" "truly_api_deployment" {
  for_each = toset(var.regions)
  region   = each.key

  //api_id      = aws_apigatewayv2_api.truly_api.id
  api_id    = aws_apigatewayv2_api.truly_api[each.key].id
  description = "truly API deployment"

  # lifecycle {
  #   create_before_destroy = true
  # }
}

