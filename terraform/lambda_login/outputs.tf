output "lambda" {
  description = "lambda execution runtime for login"
  value = aws_lambda_function.truly_lambda_login 
  
  #aws_apigatewayv2_stage.truly_stage.invoke_url
}
