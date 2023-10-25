output "invoke_url_v1" {
  value = aws_apigatewayv2_stage.truly_stage.invoke_url
}
output "invoke_url_default" {
  value = aws_apigatewayv2_stage.default_stage.invoke_url
}
output "video_in_topic" {
  value = aws_sns_topic.video_in_topic.arn
}

output "video_out_topic" {
  value = aws_sns_topic.video_out_topic.arn
}
output "video_error_topic" {
  value = aws_sns_topic.video_error_topic.arn
}
output "api_domain_name_default" {
  value = aws_apigatewayv2_domain_name.truly_api_domain_name_default.domain_name
}
output "api_domain_name_truly_version" {
  value = aws_apigatewayv2_domain_name.truly_api_domain_name_version.domain_name
}
output "notify_new_similars_topic" {
  value = aws_sns_topic.notify_new_similar_topic.arn
}

output "ses_email_ses_sender_domain" {
  value = aws_ses_domain_identity.email_ses_sender_domain.domain
}


