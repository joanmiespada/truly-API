output "lambda" {
  description = "lambda execution runtime for admin operations"
  value = aws_lambda_function.truly_lambda_admin 
}
