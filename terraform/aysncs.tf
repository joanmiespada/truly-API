// ---------- SNS video topics ------------
// start processing video
# resource "aws_sns_topic" "video_in_topic" {
#   name = "video_in_topic" # no need to add region 
#   tags = merge(local.common_tags, { service : "video api" })
# }
# // when video has been processed
# resource "aws_sns_topic" "video_out_topic" {
#   name = "video_out_topic" # no need to add region
#   tags = merge(local.common_tags, { service : "video api" })
# }

// start processing hashes and similarities
resource "aws_sns_topic" "video_in_topic" {
  name = "video_in_topic" # no need to add region 
  tags = merge(local.common_tags, { logic : "for matchapi" })
}

resource "aws_sns_topic" "video_out_topic" {
  name = "video_out_topic" # no need to add region 
  tags = merge(local.common_tags, { logic : "for matchapi" })
}
resource "aws_sns_topic" "video_error_topic" {
  name = "video_error_topic" # no need to add region 
  tags = merge(local.common_tags, { logic : "for matchapi" })
}

# // ---------- minting  ------------
# resource "aws_sqs_queue" "minting_queue" {
#   name                       = "async_minting_queue_${local.region_prefix}_${var.api_stage_version}"
#   delay_seconds              = 0
#   max_message_size           = 4096
#   message_retention_seconds  = 3600 //1h
#   visibility_timeout_seconds = 900  // 15 minutes, it needs to be aligned with lambda_mint timeout
#   receive_wait_time_seconds  = 10
#   redrive_policy = jsonencode({
#     deadLetterTargetArn = aws_sqs_queue.minting_queue_deadletter.arn,
#     maxReceiveCount     = 4
#   })
#   tags = merge(local.common_tags, {})
# }

# resource "aws_sqs_queue" "minting_queue_deadletter" {
#   name = "dead_letter_queue_mint_errors_${local.region_prefix}_${var.api_stage_version}"
#   tags = merge(local.common_tags, {})
# }

# resource "aws_cloudwatch_metric_alarm" "minting_queue_deadletter_alarm" {
#   alarm_name                = "minting_queue_deadletter_${local.region_prefix}_${var.api_stage_version}"
#   comparison_operator       = "GreaterThanThreshold"
#   evaluation_periods        = "1"
#   metric_name               = "ApproximateNumberOfMessagesVisible"
#   namespace                 = "AWS/SQS"
#   period                    = "60"
#   statistic                 = "Sum"
#   threshold                 = "0"
#   alarm_description         = "This metric monitors minting dead letter queue"
#   insufficient_data_actions = []
#   alarm_actions             = [aws_sns_topic.minting_dead_letter_topic.arn]
# }

# resource "aws_sns_topic" "minting_dead_letter_topic" {
#   name = "minting_dead_letter_topic_${local.region_prefix}_${var.api_stage_version}"
#   tags = merge(local.common_tags, {})
# }
# resource "aws_sns_topic_subscription" "minting_topic_subscription_deadletter_email" {
#   topic_arn = aws_sns_topic.minting_dead_letter_topic.arn
#   protocol  = "email"
#   endpoint  = var.email
# }


# // ---------- SNS topic ------------

# resource "aws_sns_topic" "minting_topic" {
#   name = "minting_async_topic_${local.region_prefix}_${var.api_stage_version}"
#   tags = merge(local.common_tags, {})
# }

# resource "aws_sns_topic_subscription" "mintin_async_topic_subscription" {
#   topic_arn = aws_sns_topic.minting_topic.arn
#   protocol  = "sqs"
#   endpoint  = aws_sqs_queue.minting_queue.arn
# }

# resource "aws_sqs_queue_policy" "minting_queue_policy" {
#   queue_url = aws_sqs_queue.minting_queue.id
#   policy    = <<POLICY
# {
#   "Version": "2012-10-17",
#   "Id": "sqspolicy",
#   "Statement": [
#     {
#       "Sid": "First",
#       "Effect": "Allow",
#       "Principal": "*",
#       "Action": "sqs:SendMessage",
#       "Resource": "${aws_sqs_queue.minting_queue.arn}",
#       "Condition": {
#         "ArnEquals": {
#           "aws:SourceArn": "${aws_sns_topic.minting_topic.arn}"
#         }
#       }
#     }
#   ]
# }
# POLICY
# }




# // -------- after video processing ------------


# resource "aws_sqs_queue" "after_video_queue" {
#   name                       = "after_video_queue_${local.region_prefix}_${var.api_stage_version}"
#   delay_seconds              = 0
#   max_message_size           = 4096
#   message_retention_seconds  = 3600 //1h
#   visibility_timeout_seconds = 300  // 5 minutes, it needs to be aligned with lambda_mint timeout
#   receive_wait_time_seconds  = 10
#   redrive_policy = jsonencode({
#     deadLetterTargetArn = aws_sqs_queue.after_video_queue_deadletter.arn,
#     maxReceiveCount     = 4
#   })
#   tags = merge(local.common_tags, {})
# }

# resource "aws_sqs_queue" "after_video_queue_deadletter" {
#   name = "dead_letter_queue_mint_errors_${local.region_prefix}_${var.api_stage_version}"
#   # redrive_allow_policy = jsonencode({
#   #   redrivePermission = "byQueue",
#   #   sourceQueueArns   = [aws_sqs_queue.after_video_queue.arn]
#   # })
#   tags = merge(local.common_tags, {})
# }

# resource "aws_cloudwatch_metric_alarm" "after_video_queue_deadletter_alarm" {
#   alarm_name                = "after_video_queue_deadletter_${local.region_prefix}_${var.api_stage_version}"
#   comparison_operator       = "GreaterThanThreshold"
#   evaluation_periods        = "1"
#   metric_name               = "ApproximateNumberOfMessagesVisible"
#   namespace                 = "AWS/SQS"
#   period                    = "60"
#   statistic                 = "Sum"
#   threshold                 = "0"
#   alarm_description         = "This metric monitors after_video dead letter queue"
#   insufficient_data_actions = []
#   alarm_actions             = [aws_sns_topic.after_video_dead_letter_topic.arn]
# }

# resource "aws_sns_topic" "after_video_dead_letter_topic" {
#   name = "after_video_dead_letter_topic_${local.region_prefix}_${var.api_stage_version}"
#   tags = merge(local.common_tags, {})
# }
# resource "aws_sns_topic_subscription" "after_video_topic_subscription_deadletter_email" {
#   topic_arn = aws_sns_topic.after_video_dead_letter_topic.arn
#   protocol  = "email"
#   endpoint  = var.email
# }

# resource "aws_sns_topic_subscription" "after_video_topic_subscription" {
#   topic_arn = aws_sns_topic.video_out_topic.arn
#   protocol  = "sqs"
#   endpoint  = aws_sqs_queue.after_video_queue.arn
# }


# resource "aws_sqs_queue_policy" "after_video_queue_policy" {
#   queue_url = aws_sqs_queue.after_video_queue.id
#   policy    = <<POLICY
# {
#   "Version": "2012-10-17",
#   "Id": "sqspolicy",
#   "Statement": [
#     {
#       "Sid": "First",
#       "Effect": "Allow",
#       "Principal": "*",
#       "Action": "sqs:SendMessage",
#       "Resource": "${aws_sqs_queue.after_video_queue.arn}",
#       "Condition": {
#         "ArnEquals": {
#           "aws:SourceArn": "${aws_sns_topic.video_out_topic.arn}"
#         }
#       }
#     }
#   ]
# }
# POLICY
# }

# //--------- topic to regisgter minting fails after several retries -----------
# resource "aws_sns_topic" "minting_fails_after_max_retries_topic" {
#   name = "minting_fails_after_max_retries_topic_${local.region_prefix}_${var.api_stage_version}"
#   tags = merge(local.common_tags, {})
# }

# resource "aws_cloudwatch_metric_alarm" "minting_fails_after_max_retries_alarm" {
#   alarm_name                = "minting_fails_after_max_retries_${local.region_prefix}_${var.api_stage_version}"
#   comparison_operator       = "GreaterThanThreshold"
#   evaluation_periods        = "1"
#   metric_name               = "ApproximateNumberOfMessagesVisible"
#   namespace                 = "AWS/SNS"
#   period                    = "60"
#   statistic                 = "Sum"
#   threshold                 = "0"
#   alarm_description         = "This metric monitors minting retries failed"
#   insufficient_data_actions = []
#   //alarm_actions             = [aws_sns_topic.____.arn]
# }

# resource "aws_sns_topic_subscription" "minting_fails_after_max_retries_topic_email" {
#   topic_arn = aws_sns_topic.minting_fails_after_max_retries_topic.arn
#   protocol  = "email"
#   endpoint  = var.email
# }

