
# Pre-requistes

Rust toolchain update

- rustup update
- docker-compose up -d

# Localstack status

- http://localhost:4566/health

# Create secrets

This section contains all dependencies that won't be terraformed.

- aws --endpoint-url=http://localhost:4566 --region=eu-central-1 secretsmanager create-secret --name "truly/api/secrets" --description "My test database secret created with the CLI" --secret-string file://./scripts/fake_secret.json
- aws --endpoint-url=http://localhost:4566 --region=eu-central-1 secretsmanager create-secret --name "truly/api/secret_key" --description "Storing encrypthed secret key for contract owner" --secret-string ""

- aws --endpoint-url=http://localhost:4566 --region=eu-central-1 secretsmanager get-secret-value  --secret-id "truly/api/secrets"

- aws --endpoint-url=http://localhost:4566 --region=eu-central-1 kms create-key --key-usage # at localstack don't set up anything else like ENCRYPT_DECRYPT
- aws --endpoint-url=http://localhost:4566 --region=eu-central-1 kms list-keys
- aws --endpoint-url=http://localhost:4566 --region=eu-central-1 kms describe-key --key-id <>
- aws --endpoint-url=http://localhost:4566 --region=eu-central-1 kms get-public-key --key-id <>
- aws --endpoint-url=http://localhost:4566 --region=eu-central-1 kms encrypt \
   --key-id 336d7e5e-9d0e-44c6-8ebb-2bb792bb79d0 \
   --plaintext "some important stuff" \
   --output text \
   --query CiphertextBlob \
  | base64 --decode 

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
terraform plan -var-file="variables-xxx.tfvars"
terraform apply -var-file="variables-xxx.tfvars"
terraform apply --auto-approve -var-file="variables-xxx.tfvars"
terraform destroy -var-file="variables-xxx.tfvars"

# aws KMS cypher examples

- aws kms encrypt \
   --key-id 2d460536-1dc9-436c-a97b-0bad3f8906c7  \
   --plaintext fileb://<(echo 'hi')  \
   --output text --query CiphertextBlob > deleteme.txt

- aws kms decrypt \
  --key-id 2d460536-1dc9-436c-a97b-0bad3f8906c7  \
  --ciphertext-blob fileb://<(echo 'AQICAHg4sLjNSwfVr9EthTjrQos1zT5GZj9wO1v3Dqx6F43SbgHhV3wGYKaafl7cYhhyY4foAAAAYTBfBgkqhkiG9w0BBwagUjBQAgEAMEsGCSqGSIb3DQEHATAeBglghkgBZQMEAS4wEQQMkAV6ZrBB7jA8YoGeAgEQgB7ymQRiydEFN7Q7IxPBvWa5yUTcLk6ZFFHL/oOK4YE=' | base64 -d) \
  --output text \
  --query Plaintext | base64 -d


https://janaka.dev/Simple%20example%20of%20KMS%20encrypt%20and%20decrypt%20using%20AWS%20CLI%20v2/

aws kms encrypt --endpoint-url=http://localhost:4566 --region=eu-central-1 \
--key-id 336d7e5e-9d0e-44c6-8ebb-2bb792bb79d0 \
--plaintext fileb://<(echo 'hi')  \
   --output text --query CiphertextBlob \
   | base64 --decode > deleteme.txt 

aws kms decrypt \
  --endpoint-url=http://localhost:4566 --region=eu-central-1 \
  --ciphertext-blob fileb://deleteme.txt \
  --output text \
  --query Plaintext | base64 --decode