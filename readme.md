# Pre-requistes 
Rust toolchain update 
- rustup update

# Localstack status
- http://localhost:4566/health
# Create secrets

- aws --endpoint-url=http://localhost:4566 --region=eu-central-1 secretsmanager create-secret --name "truly/api/secrets" --description "My test database secret created with the CLI" --secret-string file://./scripts/fake_secret.json
- aws --endpoint-url=http://localhost:4566 --region=eu-central-1 secretsmanager get-secret-value  --secret-id "truly/api/secrets"

# Create Tables and basic data

- aws dynamodb list-tables    --endpoint-url http://localhost:4566 
- aws dynamodb describe-table --endpoint-url http://localhost:4566 --table-name truly_users 

- NODE_ENV=development node  ./scripts/tableCreations.js --delete 
- NODE_ENV=development node  ./scripts/tableCreations.js --create
- NODE_ENV=development node  ./scripts/tableCreations.js --create -t truly_users
- NODE_ENV=development node  ./scripts/admin_user.js #create admin user

# Compile and run server_http

- cargo build --workspace  --exclude lambda_*

- ENVIRONMENT=development cargo run -p server_http

- open with postmand http://localhost:8080

# Compile and run lambdas local dev

cargo build --release --workspace --exclude server_*

Run workspace with all lambdas
- ENVIRONMENT=development cargo lambda watch
- open with postmand http://localhost:9000/lambda-url/lambda_login

https://www.cargo-lambda.info/guide/getting-started.html#step-2-create-a-new-project

- cargo lambda start
- http://localhost:9000/lambda-url/xxxx/... 

# compile lambdas for production

- cargo lambda build --release --arm64 --output-format zip --workspace  --exclude server_* --lambda-dir target/lambda_arm64

# infrastructure in productions

terraform steps: https://awstip.com/crud-operations-with-rust-on-aws-lambda-part-2-bd1feae2554b

terraform init
terraform plan
terraform apply
terraform apply --auto-approve
terraform destroy