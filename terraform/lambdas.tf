
resource "aws_iam_role" "truly_lambda_execution_role" {
  name = "truly_lambda_exec_role-${local.region_prefix}"
  assume_role_policy = file("./role_policies/assume.json")
  
  tags = merge(local.common_tags,{})
}

// -------------- Logs -------------------
resource "aws_iam_policy" "truly_lambda_logging_policy" {
  name        = "truly_lambda_logging_policy-${local.region_prefix}"
  path        = "/"
  description = "IAM policy for logging from a lambda running within truly api"

  policy = file("./role_policies/logs.json")
  
  tags = merge(local.common_tags,{})
}
resource "aws_iam_role_policy_attachment" "truly_lambda_logs" {
  role       = aws_iam_role.truly_lambda_execution_role.name
  policy_arn = aws_iam_policy.truly_lambda_logging_policy.arn
  
}
// -------------- Dynamodb -------------------
resource "aws_iam_policy" "truly_lambda_dynamodb_policy" {
  name        = "truly_lambda_dynamodb_policy-${local.region_prefix}"
  path        = "/"
  description = "IAM policy for Dynamodb from a lambda within truly api"

  policy = file("./role_policies/dynamodb.json")
  
}
resource "aws_iam_role_policy_attachment" "truly_lambda_dynamodb" {
  role       = aws_iam_role.truly_lambda_execution_role.name
  policy_arn = aws_iam_policy.truly_lambda_dynamodb_policy.arn
  
}
// -------------- S3 -------------------
resource "aws_iam_policy" "truly_lambda_S3_policy" {
  name        = "truly_lambda_S3_policy-${local.region_prefix}"
  path        = "/"
  description = "IAM policy for S3 from a lambda within truly api"

  policy = file("./role_policies/s3.json")
  
}
resource "aws_iam_role_policy_attachment" "truly_lambda_S3" {
  role       = aws_iam_role.truly_lambda_execution_role.name
  policy_arn = aws_iam_policy.truly_lambda_S3_policy.arn
}
// -------------- SNS -------------------
resource "aws_iam_policy" "truly_lambda_SNS_policy" {
  name        = "truly_lambda_SNS_policy-${local.region_prefix}"
  path        = "/"
  description = "IAM policy for SNS from a lambda within truly api"

  policy = file("./role_policies/sns.json")
}
resource "aws_iam_role_policy_attachment" "truly_lambda_SNS" {
  role       = aws_iam_role.truly_lambda_execution_role.name
  policy_arn = aws_iam_policy.truly_lambda_SNS_policy.arn
}

// -------------- X-RAY -------------------
resource "aws_iam_policy" "truly_lambda_XRAY_policy" {
  name        = "truly_lambda_XRAY_policy-${local.region_prefix}"
  path        = "/"
  description = "IAM policy for XRAY from a lambda within truly api"

  policy = file("./role_policies/x-ray.json")
}
resource "aws_iam_role_policy_attachment" "truly_lambda_XRAY" {
  role       = aws_iam_role.truly_lambda_execution_role.name
  policy_arn = aws_iam_policy.truly_lambda_XRAY_policy.arn
}

// -------------- Secrets Manager -------------------
resource "aws_iam_policy" "truly_lambda_SECRETSMAN_policy" {
  name        = "truly_lambda_SECRETSMAN_policy-${local.region_prefix}"
  path        = "/"
  description = "IAM policy for Secrets Manager from a lambda within truly api"

  policy = file("./role_policies/secretsman.json")
}
resource "aws_iam_role_policy_attachment" "truly_lambda_SECRETSMAN" {
  role       = aws_iam_role.truly_lambda_execution_role.name
  policy_arn = aws_iam_policy.truly_lambda_SECRETSMAN_policy.arn
}
// -------------- KMS -------------------
resource "aws_iam_policy" "truly_lambda_KMS_policy" {
  name        = "truly_lambda_KMS_policy-${local.region_prefix}"
  path        = "/"
  description = "IAM policy for KMS from a lambda within truly api"

  policy = file("./role_policies/kms.json")
}
resource "aws_iam_role_policy_attachment" "truly_lambda_KMS" {
  role       = aws_iam_role.truly_lambda_execution_role.name
  policy_arn = aws_iam_policy.truly_lambda_KMS_policy.arn
}
// -------------- SQS -------------------
resource "aws_iam_policy" "truly_lambda_SQS_policy" {
  name        = "truly_lambda_SQS_policy-${local.region_prefix}"
  path        = "/"
  description = "IAM policy for SQS from a lambda within truly api"

  policy = file("./role_policies/sqs.json")
}
resource "aws_iam_role_policy_attachment" "truly_lambda_SQS" {
  role       = aws_iam_role.truly_lambda_execution_role.name
  policy_arn = aws_iam_policy.truly_lambda_SQS_policy.arn
}
// -------------- QLDB -------------------
resource "aws_iam_policy" "truly_lambda_QLDB_policy" {
  name        = "truly_lambda_QLDB_policy-${local.region_prefix}"
  path        = "/"
  description = "IAM policy for QLDB from a lambda within truly api"

  policy = file("./role_policies/qldb.json")
}
resource "aws_iam_role_policy_attachment" "truly_lambda_QLDB" {
  role       = aws_iam_role.truly_lambda_execution_role.name
  policy_arn = aws_iam_policy.truly_lambda_QLDB_policy.arn
}
// -------------- execution role -------------------
resource "aws_iam_role_policy_attachment" "truly_lambda_execution_policy" {
  role       = aws_iam_role.truly_lambda_execution_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

