#topics to interact with other domains
locals {
  descriptor = "${local.common_tags.project}-${local.common_tags.environment}-${var.aws_region}"
}

// start processing hashes and similarities
resource "aws_sns_topic" "video_in_topic" {
  name = "${local.descriptor}-video_in_topic"
  tags = merge(local.common_tags, { logic : "for matchapi" })
}

resource "aws_sns_topic" "video_out_topic" {
  name = "${local.descriptor}-video_out_topic"
  tags = merge(local.common_tags, { logic : "for matchapi" })
}
resource "aws_sns_topic" "video_error_topic" {
  name = "${local.descriptor}-video_error_topic"
  tags = merge(local.common_tags, { logic : "for matchapi" })
}


