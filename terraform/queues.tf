
resource "aws_sqs_queue" "minting_queue" {
  name                      = "async_minting_queue"
  delay_seconds             = 90
  max_message_size          = 2048
  message_retention_seconds = 86400
  receive_wait_time_seconds = 10

  tags = merge(local.common_tags,{})
}

resource "aws_sqs_queue" "minting_queue_deadletter" {
  name = "dead_letter_queue_mint_errors"
  redrive_allow_policy = jsonencode({
    redrivePermission = "byQueue",
    sourceQueueArns   = [aws_sqs_queue.minting_queue.arn]
  })
}

//--------------- plug mint queue with lambda minting ----------------

resource "aws_lambda_event_source_mapping" "truly_minting" {
  event_source_arn = aws_sqs_queue.minting_queue.arn
  function_name    = aws_lambda_function.lambda_mint.arn
}