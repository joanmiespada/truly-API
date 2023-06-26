// minting queue
resource "aws_sqs_queue" "minting_queue" {
  for_each = toset(var.regions)
  region   = each.key

  name                       = "async_minting_queue"+"-${each.key}"
  delay_seconds              = 0
  max_message_size           = 4096
  message_retention_seconds  = 3600 //1h
  visibility_timeout_seconds = 300  // 5 minutes, it needs to be aligned with lambda_mint timeout
  receive_wait_time_seconds  = 10
  redrive_policy = jsonencode({
    deadLetterTargetArn = aws_sqs_queue.minting_queue_deadletter[each.key].arn,
    maxReceiveCount     = 4
  })
  tags = merge(local.common_tags, {})
}

resource "aws_sqs_queue" "minting_queue_deadletter" {
  for_each = toset(var.regions)
  region   = each.key

  name = "dead_letter_queue_mint_errors"+"-${each.key}"
  # redrive_allow_policy = jsonencode({
  #   redrivePermission = "byQueue",
  #   sourceQueueArns   = [aws_sqs_queue.minting_queue.arn]
  # })
  tags = merge(local.common_tags, {})
}

resource "aws_cloudwatch_metric_alarm" "minting_queue_deadletter_alarm" {
  for_each = toset(var.regions)
  region   = each.key

  alarm_name                = "minting_queue_deadletter"+"-${each.key}"
  comparison_operator       = "GreaterThanThreshold"
  evaluation_periods        = "1"
  metric_name               = "ApproximateNumberOfMessagesVisible"
  namespace                 = "AWS/SQS"
  period                    = "60"
  statistic                 = "Sum"
  threshold                 = "0"
  alarm_description         = "This metric monitors minting dead letter queue"
  insufficient_data_actions = []
  alarm_actions             = [aws_sns_topic.minting_dead_letter_topic[each.key].arn]
}

resource "aws_sns_topic" "minting_dead_letter_topic" {
  for_each = toset(var.regions)
  region   = each.key

  name = "minting_dead_letter_topic"+"-${each.key}"
  tags = merge(local.common_tags, {})
}
resource "aws_sns_topic_subscription" "minting_topic_subscription_deadletter_email" {
  for_each = toset(var.regions)
  region   = each.key

  topic_arn = aws_sns_topic.minting_dead_letter_topic[each.key].arn
  protocol  = "email"
  endpoint  = var.email
}


// ---------- SNS topic ------------

resource "aws_sns_topic" "minting_topic" {
  for_each = toset(var.regions)
  region   = each.key

  name = "minting_async_topic"+"-${each.key}"
  tags = merge(local.common_tags, {})
}

resource "aws_sns_topic_subscription" "mintin_async_topic_subscription" {
  for_each = toset(var.regions)
  region   = each.key

  topic_arn = aws_sns_topic.minting_topic[each.key].arn
  protocol  = "sqs"
  endpoint  = aws_sqs_queue.minting_queue[each.key].arn
}

# resource "aws_sqs_queue_policy" "minting_queue_policy" {
#   queue_url = aws_sqs_queue.minting_queue.id
#   policy    = file("./role_policies/async_mint_queue.json")
#}

resource "aws_sqs_queue_policy" "minting_queue_policy" {
  for_each = toset(var.regions)
  region   = each.key

  queue_url = aws_sqs_queue.minting_queue[each.key].id
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
      "Resource": "${aws_sqs_queue.minting_queue[each.key].arn}",
      "Condition": {
        "ArnEquals": {
          "aws:SourceArn": "${aws_sns_topic.minting_topic[each.key].arn}"
        }
      }
    }
  ]
}
POLICY
}


// ---------- SNS video topics ------------
// start processing video
resource "aws_sns_topic" "video_in_topic" {
  for_each = toset(var.regions)
  region   = each.key

  name = "video_in_topic"+"-${each.key}"
  tags = merge(local.common_tags, { service : "video api" })
}
// when video has been processed
resource "aws_sns_topic" "video_out_topic" {
  for_each = toset(var.regions)
  region   = each.key

  name = "video_out_topic"+"-${each.key}"
  tags = merge(local.common_tags, { service : "video api" })
}

// -------- after video processing ------------


resource "aws_sqs_queue" "after_video_queue" {
  for_each = toset(var.regions)
  region   = each.key

  name                       = "after_video_queue"+"-${each.key}"
  delay_seconds              = 0
  max_message_size           = 4096
  message_retention_seconds  = 3600 //1h
  visibility_timeout_seconds = 300  // 5 minutes, it needs to be aligned with lambda_mint timeout
  receive_wait_time_seconds  = 10
  redrive_policy = jsonencode({
    deadLetterTargetArn = aws_sqs_queue.after_video_queue_deadletter[each.key].arn,
    maxReceiveCount     = 4
  })
  tags = merge(local.common_tags, {})
}

resource "aws_sqs_queue" "after_video_queue_deadletter" {
  for_each = toset(var.regions)
  region   = each.key

  name = "dead_letter_queue_mint_errors"+"-${each.key}"
  # redrive_allow_policy = jsonencode({
  #   redrivePermission = "byQueue",
  #   sourceQueueArns   = [aws_sqs_queue.after_video_queue.arn]
  # })
  tags = merge(local.common_tags, {})
}

resource "aws_cloudwatch_metric_alarm" "after_video_queue_deadletter_alarm" {
  for_each = toset(var.regions)
  region   = each.key

  alarm_name                = "after_video_queue_deadletter"+"-${each.key}"
  comparison_operator       = "GreaterThanThreshold"
  evaluation_periods        = "1"
  metric_name               = "ApproximateNumberOfMessagesVisible"
  namespace                 = "AWS/SQS"
  period                    = "60"
  statistic                 = "Sum"
  threshold                 = "0"
  alarm_description         = "This metric monitors after_video dead letter queue"
  insufficient_data_actions = []
  alarm_actions             = [aws_sns_topic.after_video_dead_letter_topic[each.key].arn]
}

resource "aws_sns_topic" "after_video_dead_letter_topic" {
  for_each = toset(var.regions)
  region   = each.key

  name = "after_video_dead_letter_topic"+"-${each.key}"
  tags = merge(local.common_tags, {})
}
resource "aws_sns_topic_subscription" "after_video_topic_subscription_deadletter_email" {
  for_each = toset(var.regions)
  region   = each.key

  topic_arn = aws_sns_topic.after_video_dead_letter_topic[each.key].arn
  protocol  = "email"
  endpoint  = var.email
}

resource "aws_sns_topic_subscription" "after_video_topic_subscription" {
  for_each = toset(var.regions)
  region   = each.key

  topic_arn = aws_sns_topic.video_out_topic[each.key].arn
  protocol  = "sqs"
  endpoint  = aws_sqs_queue.after_video_queue[each.key].arn
}


resource "aws_sqs_queue_policy" "after_video_queue_policy" {
  for_each = toset(var.regions)
  region   = each.key

  queue_url = aws_sqs_queue.after_video_queue[each.key].id
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
      "Resource": "${aws_sqs_queue.after_video_queue[each.key].arn}",
      "Condition": {
        "ArnEquals": {
          "aws:SourceArn": "${aws_sns_topic.video_out_topic[each.key].arn}"
        }
      }
    }
  ]
}
POLICY
}

//--------- topic to regisgter minting fails after several retries -----------
resource "aws_sns_topic" "minting_fails_after_max_retries_topic" {
  for_each = toset(var.regions)
  region   = each.key

  name = "minting_fails_after_max_retries_topic"+"-${each.key}"
  tags = merge(local.common_tags, {})
}

resource "aws_cloudwatch_metric_alarm" "minting_fails_after_max_retries_alarm" {
  for_each = toset(var.regions)
  region   = each.key

  alarm_name                = "minting_fails_after_max_retries"+"-${each.key}"
  comparison_operator       = "GreaterThanThreshold"
  evaluation_periods        = "1"
  metric_name               = "ApproximateNumberOfMessagesVisible"
  namespace                 = "AWS/SNS"
  period                    = "60"
  statistic                 = "Sum"
  threshold                 = "0"
  alarm_description         = "This metric monitors minting retries failed"
  insufficient_data_actions = []
  //alarm_actions             = [aws_sns_topic.____.arn]
}

resource "aws_sns_topic_subscription" "minting_fails_after_max_retries_topic_email" {
  for_each = toset(var.regions)
  region   = each.key

  topic_arn = aws_sns_topic.minting_fails_after_max_retries_topic[each.key].arn
  protocol  = "email"
  endpoint  = var.email
}

