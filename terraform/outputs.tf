output "invoke_url_aws" {
  value = aws_apigatewayv2_stage.truly_stage.invoke_url
}
# output "video_in_topic" {
#   value = aws_sns_topic.video_in_topic.arn
# }

# output "video_out_topic" {
#   value = aws_sns_topic.video_out_topic.arn
# }

output "api_domain_name_default" {
  value = aws_apigatewayv2_domain_name.truly_api_domain_name_default.domain_name
}
output "api_domain_name_truly_version" {
  value = aws_apigatewayv2_domain_name.truly_api_domain_name_version.domain_name
}

# output "api_gateway_url_localstack_1" {
#   description = "The URL of the deployed API Gateway on LocalStack option1"
#   value       = "http://localhost:4566/restapis/${aws_apigatewayv2_api.truly_api.id}/${var.api_stage_version}/_user_request_/<path>"
# }
# output "api_gateway_url_localstack_2" {
#   description = "The URL of the deployed API Gateway on LocalStack option2"
#   value       = "http://${aws_apigatewayv2_api.truly_api.id}.execute-api.localhost.localstack.cloud:4566/${var.api_stage_version}/<path>"
# }

