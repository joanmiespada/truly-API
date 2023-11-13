locals {
  
  tags = merge(var.common_tags, { "logic" : "${var.service_name}" })
}

resource "aws_sqs_queue" "after_hash_queue" {
  name                       = local.lambda_name_descriptor 
  delay_seconds              = 0
  max_message_size           = 4096
  message_retention_seconds  = 3600 //1h
  visibility_timeout_seconds = 300  // 5 minutes, it needs to be aligned with lambda_mint timeout
  receive_wait_time_seconds  = 10
  redrive_policy = jsonencode({
    deadLetterTargetArn = aws_sqs_queue.after_hash_queue_deadletter.arn,
    maxReceiveCount     = 4
  })
  tags = local.tags
}

resource "aws_sqs_queue" "after_hash_queue_deadletter" {
  name = "${local.lambda_name_descriptor}-dead_letter" 
  # redrive_allow_policy = jsonencode({
  #   redrivePermission = "byQueue",
  #   sourceQueueArns   = [aws_sqs_queue.after_video_queue.arn]
  # })
  tags = local.tags
}

resource "aws_cloudwatch_metric_alarm" "after_hash_queue_deadletter_alarm" {
  alarm_name                = "${local.lambda_name_descriptor}-dead_letter"
  comparison_operator       = "GreaterThanThreshold"
  evaluation_periods        = "1"
  metric_name               = "ApproximateNumberOfMessagesVisible"
  namespace                 = "AWS/SQS"
  period                    = "60"
  statistic                 = "Sum"
  threshold                 = "0"
  alarm_description         = "This metric monitors after_hash dead letter queue"
  insufficient_data_actions = []
  alarm_actions             = [aws_sns_topic.after_hash_dead_letter_topic.arn]
}

resource "aws_sns_topic" "after_hash_dead_letter_topic" {
  name = "${local.lambda_name_descriptor}-dead_letter"
  tags = local.tags
}
resource "aws_sns_topic_subscription" "after_hash_topic_subscription_deadletter_email" {
  topic_arn = aws_sns_topic.after_hash_dead_letter_topic.arn
  protocol  = "email"
  endpoint  = var.email
}

resource "aws_sns_topic_subscription" "after_hash_topic_subscription" {
  topic_arn = var.video_out_topic_arn
  protocol  = "sqs"
  endpoint  = aws_sqs_queue.after_hash_queue.arn
}

resource "aws_sqs_queue_policy" "download_queue_policy" {
  queue_url = aws_sqs_queue.after_hash_queue.id
  policy    = <<POLICY
{
  "Version": "2012-10-17",
  "Id": "sqspolicy",
  "Statement": [
    {
      "Sid": "First",
      "Effect": "Allow",
      "Principal": "*",
      "Action": "sqs:SendMessage",
      "Resource": "${aws_sqs_queue.after_hash_queue.arn}",
      "Condition": {
        "ArnEquals": {
          "aws:SourceArn": "${var.video_out_topic_arn}"
        }
      }
    }
  ]
}
POLICY
}



