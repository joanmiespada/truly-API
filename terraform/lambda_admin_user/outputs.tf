output "lambda" {
  description = "lambda execution runtime for admin user operations"
  value = aws_lambda_function.truly_lambda_admin_user 
}
