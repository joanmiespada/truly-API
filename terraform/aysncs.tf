
resource "aws_sqs_queue" "minting_queue" {
  name                      = "async_minting_queue"
  delay_seconds             = 0
  max_message_size          = 4096
  message_retention_seconds = 3600 //1h
  visibility_timeout_seconds = 300 // 5 minutes, it needs to be aligned with lambda_mint timeout
  receive_wait_time_seconds = 10
  redrive_policy = jsonencode({
    deadLetterTargetArn = aws_sqs_queue.minting_queue_deadletter.arn,
    maxReceiveCount     = 4
  })
  tags = merge(local.common_tags, {})
}

resource "aws_sqs_queue" "minting_queue_deadletter" {
  name = "dead_letter_queue_mint_errors"
  # redrive_allow_policy = jsonencode({
  #   redrivePermission = "byQueue",
  #   sourceQueueArns   = [aws_sqs_queue.minting_queue.arn]
  # })
  tags = merge(local.common_tags, {})
}



// ---------- SNS topic ------------

resource "aws_sns_topic" "minting_topic" {
  name = "minting_async_topic"
  tags = merge(local.common_tags, {})
}

resource "aws_sns_topic_subscription" "mintin_async_topic_subscription" {
  topic_arn = aws_sns_topic.minting_topic.arn
  protocol  = "sqs"
  endpoint  = aws_sqs_queue.minting_queue.arn
}

resource "aws_sqs_queue_policy" "minting_queue_policy" {
  queue_url = aws_sqs_queue.minting_queue.id
  policy    = file("./role_policies/async_mint_queue.json")

}

// TO BE DELETED!!!
# resource "aws_sns_topic_subscription" "mintin_async_topic_subscription_debug_email" {
#   topic_arn = aws_sns_topic.minting_topic.arn
#   protocol  = "email"
#   endpoint  = "joanmi@espada.cat"
# }


// ---------- SNS video topics ------------
// start processing video
resource "aws_sns_topic" "video_in_topic" {
   name = "video_in_topic"
   tags = merge(local.common_tags, { service:"video"  })
}
// when video has been processed
resource "aws_sns_topic" "video_out_topic" {
   name = "video_out_topic"
   tags = merge(local.common_tags, { service:"video"  })
}

