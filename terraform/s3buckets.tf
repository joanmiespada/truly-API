resource "aws_s3_bucket" "truly-cam-public-upload" {
  bucket = "truly-cam-public-upload"

    lifecycle_rule {
      id =  "remove_old_files"
      enabled = true
      expiration {
        days = 1
      }
    }

  tags = merge(local.common_tags,{})
}

resource "aws_s3_bucket_public_access_block" "truly-cam-public-upload" {
  bucket = aws_s3_bucket.truly-cam-public-upload.id

  block_public_acls       = false
  block_public_policy     = false
  ignore_public_acls      = false
  restrict_public_buckets = false
  
}


resource "aws_s3_bucket" "truly-cam-private-exchange-operations" {
  bucket = "truly-cam-private-exchange-operations"

    lifecycle_rule {
      id =  "remove_old_files"
      enabled = true
      expiration {
        days = 1
      }
    }

  tags = merge(local.common_tags,{})
}


resource "aws_s3_bucket_public_access_block" "truly-cam-public-upload-block" {
  bucket = aws_s3_bucket.truly-cam-private-exchange-operations.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
  
}

