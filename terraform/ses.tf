
//This is for SQS subscriptions for debuging purposes
resource "aws_ses_email_identity" "email_ses_sender" {
  email = var.ses_from_email
}


resource "aws_ses_domain_identity" "email_ses_sender_domain" {
  domain = var.ses_domain
}

resource "aws_iam_user" "smtp_user" {
  name = "smtp_user"
  tags = local.common_tags
}

resource "aws_iam_access_key" "smtp_user" {
  user = aws_iam_user.smtp_user.name
}

resource "aws_iam_policy" "truly_lambda_SES_policy" {
  name        = "truly_lambda_SES_policy-${local.region_prefix}"
  path        = "/"
  description = "IAM policy for SES from a lambda within truly api"

  policy = file("./role_policies/ses.json")
}

resource "aws_iam_policy" "ses_sender" {
  name        = "ses_sender"
  description = "Allows sending of e-mails via Simple Email Service"
  policy      = aws_iam_policy.truly_lambda_SES_policy.policy
  # policy      = data.aws_iam_policy_document.ses_sender.json
}

resource "aws_iam_user_policy_attachment" "user-ses-policy-attach" {
  user       = aws_iam_user.smtp_user.name
  policy_arn = aws_iam_policy.ses_sender.arn
}

output "smtp_username" {
  value = aws_iam_access_key.smtp_user.id
}

output "smtp_password" {
  value = aws_iam_access_key.smtp_user.ses_smtp_password_v4
  sensitive = true
}


resource "aws_secretsmanager_secret" "smtp_secret" {
  name = "truly_api_smtp2"
  depends_on = [
    aws_iam_user_policy_attachment.user-ses-policy-attach
  ]
  tags = local.common_tags
}

resource "aws_secretsmanager_secret_version" "smtp_secret_version" {
  secret_id     = aws_secretsmanager_secret.smtp_secret.id
  secret_string = "{\"username\":\"${aws_iam_access_key.smtp_user.id}\",\"password\":\"${aws_iam_access_key.smtp_user.ses_smtp_password_v4}\"}"
  
}