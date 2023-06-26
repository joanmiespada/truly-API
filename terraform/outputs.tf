output "invoke_url" {
  //value = aws_apigatewayv2_stage.truly_stage.invoke_url
  value       = { for region, value in aws_apigatewayv2_stage.truly_stage: region => value.invoke_url }
}
output "video_in_topic" {
  //value = aws_sns_topic.video_in_topic.arn
  value       = { for region, value in aws_sns_topic.video_in_topic: region => value.arn }
}

output "video_out_topic" {
  //value = aws_sns_topic.video_out_topic.arn
  value       = { for region, value in aws_sns_topic.video_out_topic: region => value.arn }
}

output "api" {
  //value = aws_apigatewayv2_domain_name.truly_api_domain_name.domain_name
  value       = { for region, value in aws_apigatewayv2_domain_name.truly_api_domain_name: region => value.domain_name }
}
