#!/bin/bash
architecture='aarch64-linux-gnu'
path_base='/Users/joanmiquelespadasabat/Projects/tron/API/cross-compile/openssl/'${architecture}

export OPENSSL_LIB_DIR=${path_base}/lib
export OPENSSL_INCLUDE_DIR=${path_base}/include

folder = "target/lambda_${architecture}"

cargo lambda build --release --arm64 --output-format zip --workspace  --exclude server_* --exclude truly_cli --lambda-dir $folder

cd terraform

export TF_VAR_lambda_deploy_folder="../${folder}/"
terraform plan -var-file="variables-stage.tfvars"
terraform apply -var-file="variables-stage.tfvars" --auto-approve

# terraform plan -var-file="variables-prod.tfvars"
#terraform apply -var-file="variables-prod.tfvars" --auto-approve

cd ..
