resource "aws_ses_email_identity" "email_ses_sender" {
  email = var.email
}