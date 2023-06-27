//---------------- domain ----------------------------


data "aws_route53_zone" "truly_zone" {
  name         = var.dns_base
  private_zone = false
}

resource "aws_apigatewayv2_domain_name" "truly_api_domain_name" {
  for_each = toset(var.regions)
  provider = aws.eu_west_1   #aws[each.key]

  domain_name = format("%s-%s.%s", var.dns_prefix, each.key, var.dns_base)
  domain_name_configuration {
    certificate_arn = aws_acm_certificate_validation.cert_multi_region[each.key].certificate_arn
    endpoint_type   = "REGIONAL"
    security_policy = "TLS_1_2"
  }
}

# resource "aws_route53_record" "truly_zone_record_A" {
#   for_each = toset(var.regions)

#   zone_id = data.aws_route53_zone.truly_zone.zone_id
#   name    = format("%s-%s.%s", var.dns_prefix, each.key, var.dns_base)
#   type    = "A"

#   alias {
#     name                   = aws_apigatewayv2_domain_name.truly_api_domain_name[each.key].domain_name_configuration[0].target_domain_name
#     zone_id                = aws_apigatewayv2_domain_name.truly_api_domain_name[each.key].domain_name_configuration[0].hosted_zone_id
#     evaluate_target_health = false
#   }
# }

resource "aws_apigatewayv2_api_mapping" "map_dns_agigateway" {
  for_each    = toset(var.regions)
  api_id      = aws_apigatewayv2_api.truly_api[each.key].id
  domain_name = aws_apigatewayv2_domain_name.truly_api_domain_name[each.key].domain_name
  stage       = aws_apigatewayv2_stage.truly_stage[each.key].name
  depends_on  = [aws_apigatewayv2_domain_name.truly_api_domain_name, aws_apigatewayv2_api.truly_api]
}

# resource "aws_route53_record" "latency" {
#   for_each = toset(var.regions)

#   zone_id        = data.aws_route53_zone.truly_zone.zone_id
#   name           = format("%s.%s", var.dns_prefix, var.dns_base)
#   type           = "A"
#   set_identifier = each.key

#   latency_routing_policy {
#     region = each.key
#   }

#   alias {
#     name                   = aws_apigatewayv2_domain_name.truly_api_domain_name[each.key].domain_name_configuration[0].target_domain_name
#     zone_id                = aws_apigatewayv2_domain_name.truly_api_domain_name[each.key].domain_name_configuration[0].hosted_zone_id
#     evaluate_target_health = false
#   }
# }


resource "aws_route53_traffic_policy" "geoloc_policy" {
  name = "dns_geoloc_policy"

  document = jsonencode({
    AWSPolicyFormatVersion = "2015-10-01"
    RecordType             = "A"
    Endpoints              = [
      for region in var.regions : {
        Id   = replace(region, "-", "")
        Type = "aws:region"
        Value = region
      }
    ]
    Rules = [
      {
        Name     = "Geoproximity"
        Type     = "geoproximity"
        Primary  = replace(var.regions[0], "-", "")
        Additional = {
          for region in slice(var.regions, 1, length(var.regions)) : replace(region, "-", "") => {
            Measure = "Bias"
            Value   = 50
          }
        }
      }
    ]
  })
}

resource "aws_route53_record" "truly_zone_record_A" {
  for_each = toset(var.regions)

  zone_id = data.aws_route53_zone.truly_zone.zone_id
  name    = format("%s.%s",var.dns_prefix,var.dns_base)
  type    = "A"

  alias {
    name                   = aws_apigatewayv2_domain_name.truly_api_domain_name[each.key].domain_name_configuration[0].target_domain_name
    zone_id                = aws_apigatewayv2_domain_name.truly_api_domain_name[each.key].domain_name_configuration[0].hosted_zone_id
    evaluate_target_health = false
  }

  set_identifier = each.key
  ttl            = 300

  traffic_policy_id = aws_route53_traffic_policy.geoloc_policy.id
}