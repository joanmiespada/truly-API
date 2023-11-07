
variable "service_name" {
  type    = string
}

variable "common_tags" {}
#variable "resource_logs" {}
#variable "resource_dynamodb" {}
#variable "resource_xray" {}
#variable "resource_secretsman" {}
#variable "resource_kms" {}
#variable "resource_sqs" {}
#variable "resource_sns" {}
variable "role" {
}
variable "environment_flag" {
  type = string
}
#variable "trace_log" {}
variable "rust_log" {
  type = string
}

variable "kms_cypher_owner" {
  type      = string
  description = "kms cypher for secret keys"
}

# variable "dead_letter_queue_mint" {
#   type      = string
#   description = "queue url to send errors when minting"
# }
# variable "minting_async_topic_arn" {
#   type      = string
#   description = "topic arn to mint async"
# }

variable "rust_backtrace" {
  type= string
}

# variable "video_in_topic" {
#   type = string
#   description = "topic to be connectec with other dependencies, video processing triggers"
# }

# variable "video_out_topic" {
#   type = string
#   description = "topic to be connected with other dependencies, video processing results" 
# }

# variable minting_fails_topic_arn {
#   type = string
#   description = "topic to register when miting fails after several retries"
# }
variable "aws_region" {
  type    = string
}
variable "api_stage_version" {
  type = string
}
variable "architectures" {
  type    = list(string)
}

variable "hashes_similarities_arn" {
  type    = string
  description = "topic where hash calculation and similarities will be triggered"
}

variable "matchapi_endpoint" {
  type = string
  description = "url where matchapi endpoint is"
  
}

variable "ecr_image" {
  type = string
  description = "ecr repo where I must pull the image base"
}

variable "trace_level" {
  type=string
}

variable "url_base_permanent_images" {
  type=string
}

variable "smtp_server" {
  type = string
  description = "email server for sending emails"
}

variable "smtp_from" {
  type = string
}

variable "smtp_server" {
  type = string
}


