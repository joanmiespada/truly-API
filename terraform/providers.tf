# provider "aws" {
#   //region = var.aws_region
#   profile = "personal"
# }

# provider "aws" {
#   alias = "default"
#   //region = var.aws_region
#   profile = "personal"
#}

provider "aws" {
  region = var.aws_region
  profile = "truly"
}

provider "aws" {
  alias = "default"
  region = var.aws_region
  profile = "truly"
}

provider "aws" { #only for certificates used by dns
  alias  = "useast"
  region = "us-east-1"
  profile = "personal"
}

provider "aws" { #only for certificates used by dns
  alias   = "us_east_1"
  region  = "us-east-1"
  profile = "truly"
}

# provider "aws" { #only for certificates used by dns
#   alias   = "eu_central_1"
#   region  = "eu-central-1"
#   profile = "truly"
# }
# provider "aws" { #only for certificates used by dns
#   alias   = "eu_west_1"
#   region  = "eu-west-1"
#   profile = "truly"
# }
# provider "aws" { #only for certificates used by dns
#   alias   = "ap_northeast_1"
#   region  = "ap-northeast-1"
#   profile = "truly"
#}

# provider "aws" {
#   for_each = toset(var.regions)
#   region   = each.key
  
#   alias    = each.key
#   profile = "personal"
# }