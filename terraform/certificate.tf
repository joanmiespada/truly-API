
data "aws_route53_zone" "selected" {
  name         = "truly.video"
  private_zone = false
}

resource "aws_acm_certificate" "cert" {
 # provider          = aws.useast
  domain_name       = "*.truly.video"
  validation_method = "DNS"
}

resource "aws_acm_certificate_validation" "cert" {
  #provider          = aws.default
  certificate_arn = aws_acm_certificate.cert.arn

  validation_record_fqdns = [for record in aws_acm_certificate.cert.domain_validation_options : record.resource_record_name]
}

resource "aws_route53_record" "validation" {
  for_each = {
    for dvo in aws_acm_certificate.cert.domain_validation_options : dvo.domain_name => {
      name   = dvo.resource_record_name
      record = dvo.resource_record_value
      type   = dvo.resource_record_type
    }
  }

  allow_overwrite = true
  name            = each.value.name
  records         = [each.value.record]
  ttl             = 60
  type            = each.value.type
  zone_id         = data.aws_route53_zone.selected.zone_id
}
