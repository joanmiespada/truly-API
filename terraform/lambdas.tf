
resource "aws_iam_role" "truly_lambda_execution_role" {
  assume_role_policy = file("./role_policies/assume.json")
  
  tags = merge(local.common_tags,{})
}

// -------------- Logs -------------------
resource "aws_iam_policy" "truly_lambda_logging_policy" {
  name        = "truly_lambda_logging_policy"
  path        = "/"
  description = "IAM policy for logging from a lambda running within truly app"

  policy = file("./role_policies/logs.json")
  
  tags = merge(local.common_tags,{})
}
resource "aws_iam_role_policy_attachment" "truly_lambda_logs" {
  role       = aws_iam_role.truly_lambda_execution_role.name
  policy_arn = aws_iam_policy.truly_lambda_logging_policy.arn
  
}
// -------------- Dynamodb -------------------
resource "aws_iam_policy" "truly_lambda_dynamodb_policy" {
  name        = "truly_lambda_dynamodb_policy"
  path        = "/"
  description = "IAM policy for Dynamodb from a lambda within truly app"

  policy = file("./role_policies/dynamodb.json")
  
}
resource "aws_iam_role_policy_attachment" "truly_lambda_dynamodb" {
  role       = aws_iam_role.truly_lambda_execution_role.name
  policy_arn = aws_iam_policy.truly_lambda_dynamodb_policy.arn
  
}
// -------------- S3 -------------------
resource "aws_iam_policy" "truly_lambda_S3_policy" {
  name        = "truly_lambda_S3_policy"
  path        = "/"
  description = "IAM policy for S3 from a lambda within truly app"

  policy = file("./role_policies/s3.json")
  
}
resource "aws_iam_role_policy_attachment" "truly_lambda_S3" {
  role       = aws_iam_role.truly_lambda_execution_role.name
  policy_arn = aws_iam_policy.truly_lambda_S3_policy.arn
}
// -------------- SNS -------------------
resource "aws_iam_policy" "truly_lambda_SNS_policy" {
  name        = "truly_lambda_SNS_policy"
  path        = "/"
  description = "IAM policy for SNS from a lambda within truly app"

  policy = file("./role_policies/sns.json")
}
resource "aws_iam_role_policy_attachment" "truly_lambda_SNS" {
  role       = aws_iam_role.truly_lambda_execution_role.name
  policy_arn = aws_iam_policy.truly_lambda_SNS_policy.arn
}

// -------------- X-RAY -------------------
resource "aws_iam_policy" "truly_lambda_XRAY_policy" {
  name        = "truly_lambda_XRAY_policy"
  path        = "/"
  description = "IAM policy for XRAY from a lambda within truly app"

  policy = file("./role_policies/x-ray.json")
}
resource "aws_iam_role_policy_attachment" "truly_lambda_XRAY" {
  role       = aws_iam_role.truly_lambda_execution_role.name
  policy_arn = aws_iam_policy.truly_lambda_XRAY_policy.arn
}

// -------------- execution role -------------------
resource "aws_iam_role_policy_attachment" "truly_lambda_execution_policy" {
  role       = aws_iam_role.truly_lambda_execution_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

