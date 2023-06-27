#!/bin/bash
architecture='aarch64-linux-gnu'
path_base='/Users/joanmiquelespadasabat/Projects/tron/API/cross-compile/openssl/'${architecture}

export OPENSSL_LIB_DIR=${path_base}/lib
export OPENSSL_INCLUDE_DIR=${path_base}/include

folder="target/lambda_${architecture}"

echo 'compiling lambdas...'
cargo lambda build --release --arm64 --output-format zip --workspace  --exclude server_* --exclude truly_cli --lambda-dir $folder

if [ $? -ne 0 ]; then
    echo "Compilation failed, aborting."
    exit 1
else
    echo "Compilation completed."
fi

cd terraform

echo "Terraforming..."
export TF_VAR_lambda_deploy_folder="../${folder}/"
export TF_VAR_aws_region="eu-c"
terraform plan -var-file="variables-stage.tfvars"

#terraform apply -var-file="variables-stage.tfvars" --auto-approve

# terraform plan -var-file="variables-prod.tfvars"
#terraform apply -var-file="variables-prod.tfvars" --auto-approve
if [ $? -ne 0 ]; then
    echo "Terraform failed, aborting."
    exit 1
else
    echo "Terraform completed."
fi
cd ..
