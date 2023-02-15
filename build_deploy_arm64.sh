#!/bin/bash

cargo lambda build --release --arm64 --output-format zip --workspace  --exclude server_* --exclude manual_dep --lambda-dir target/lambda_arm64

cd terraform

terraform plan -var-file="variables-stage.tfvars"
terraform apply -var-file="variables-stage.tfvars" --auto-approve

terraform plan -var-file="variables-prod.tfvars"
terraform apply -var-file="variables-prod.tfvars" --auto-approve

cd ..
