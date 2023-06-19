#!/bin/bash
architecture='aarch64-linux-gnu'
path_base='/Users/joanmiquelespadasabat/Projects/tron/API/cross-compile/openssl/'${architecture}

export OPENSSL_LIB_DIR=${path_base}/lib
export OPENSSL_INCLUDE_DIR=${path_base}/include

cargo lambda build --release --arm64 --output-format zip --workspace  --exclude server_* --exclude truly_cli --lambda-dir target/lambda_arm64

cd terraform

terraform plan -var-file="variables-stage.tfvars"
terraform apply -var-file="variables-stage.tfvars" --auto-approve

# terraform plan -var-file="variables-prod.tfvars"
#terraform apply -var-file="variables-prod.tfvars" --auto-approve

cd ..
