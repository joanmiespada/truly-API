output "lambda" {
  description = "lambda execution runtime for user operations"
  value = aws_lambda_function.truly_lambda_user 
  
}
