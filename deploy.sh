cargo lambda build --release --arm64 --output-format zip --workspace  --exclude server_*
cd terraform
terraform plan
terraform apply --auto-approve
cd ..
