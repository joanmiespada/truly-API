output "invoke_url" {
  value = aws_apigatewayv2_stage.truly_stage.invoke_url
}
output "video_in_topic" {
  value = aws_sns_topic.video_in_topic.arn
}

output "video_out_topic" {
  value = aws_sns_topic.video_out_topic.arn
}

output "api" {
  value = aws_apigatewayv2_domain_name.truly_api_domain_name.domain_name
}
