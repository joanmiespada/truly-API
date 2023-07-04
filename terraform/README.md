
# Terraform configurations

## Create workspaces

Inside ./terraform folder run these commands:

´´´bash
terraform workspace new localstack
terraform workspace new stage
terraform workspace new prod
´´´

## Bootstrap terraform's dependencies

´´´bash
terraform init
´´´
