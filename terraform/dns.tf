//---------------- domain ----------------------------

resource "aws_apigatewayv2_domain_name" "truly_api_domain_name" {
  domain_name = format("%s%s",var.dns, ".truly.video")
  domain_name_configuration {
    certificate_arn = aws_acm_certificate_validation.cert.certificate_arn
    endpoint_type   = "REGIONAL"
    security_policy = "TLS_1_2"
  }
}

data "aws_route53_zone" "truly_zone" {
  name         = "truly.video"
  private_zone = false
}

resource "aws_route53_record" "truly_zone_record_A" {
  zone_id = data.aws_route53_zone.truly_zone.zone_id
  name    = format("%s%s",var.dns,".truly.video")
  type    = "A"

  alias {
    name                   = aws_apigatewayv2_domain_name.truly_api_domain_name.domain_name_configuration[0].target_domain_name
    zone_id                = aws_apigatewayv2_domain_name.truly_api_domain_name.domain_name_configuration[0].hosted_zone_id
    evaluate_target_health = false
  }
}

resource "aws_apigatewayv2_api_mapping" "map_dns_agigateway" {
  api_id      = aws_apigatewayv2_api.truly_api.id
  domain_name = aws_apigatewayv2_domain_name.truly_api_domain_name.domain_name
  stage       = aws_apigatewayv2_stage.truly_stage.name
  depends_on  = [aws_apigatewayv2_domain_name.truly_api_domain_name, aws_apigatewayv2_api.truly_api]
}