#!/bin/bash
cargo lambda build --release --arm64 --output-format zip --workspace  --exclude server_* --exclude command_* --lambda-dir target/lambda_arm64
cd terraform
terraform plan
terraform apply --auto-approve
cd ..
