
- aws dynamodb list-tables --endpoint-url http://localhost:8000 --region=local

- NODE_ENV=development node  ./scripts/tableCreations.js --delete 
- NODE_ENV=development node  ./scripts/tableCreations.js --create
- NODE_ENV=development node  ./scripts/admin_user.js #create admin user

- cargo build
- cargo run

https://www.cargo-lambda.info/guide/getting-started.html#step-2-create-a-new-project

- cargo lambda start
- http://localhost:9000/lambda-url/api/... 

- cargo lambda build --release
- cargo lambda build --release --arm64
- cargo lambda deploy --iam-role XXXXXX  --http

Rust toolchain update 
- rustup update

Run workspace with all lambdas
ENVIRONMENT=development cargo lambda start

ENVIRONMENT=development cargo run -p server_http

terraform steps: https://awstip.com/crud-operations-with-rust-on-aws-lambda-part-2-bd1feae2554b

cargo lambda build --release --all-features --arm64 --output-format zip
cargo lambda build --release --arm64 --output-format zip --workspace  --exclude server_http

terraform init
terraform plan
terraform apply
terraform apply --auto-approve
terraform destroy