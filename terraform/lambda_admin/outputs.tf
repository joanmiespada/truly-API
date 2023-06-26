# output "lambda" {
#   description = "lambda execution runtime for admin operations"
#   value = aws_lambda_function.truly_lambda_admin 
# }

# output "lambda_ids" {
#   description = "The names (IDs) of the lambdas"
#   value       = { for region, lambda in aws_lambda_function.truly_lambda_admin : region => lambda.id }
# }

# output "lambda_arns" {
#   description = "The ARNs of the lambdas"
#   value       = { for region, lambda in aws_lambda_function.truly_lambda_admin : region => lambda.invoke_arn }
# }
output "lambdas" {
  description = "The ARNs of the lambdas in each reagion"
  value       = { for region, lambda in aws_lambda_function.truly_lambda_admin : region => lambda }
}