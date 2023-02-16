output "lambda" {
  description = "lambda execution runtime for login"
  value = aws_lambda_function.truly_lambda_login 
}
