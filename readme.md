
- aws dynamodb list-tables --endpoint-url http://localhost:8000 --region=local

- NODE_ENV=development node   tableCreations.js --create
- NODE_ENV=development node   tableCreations.js --delete 


- cargo build
- cargo run

https://www.cargo-lambda.info/guide/getting-started.html#step-2-create-a-new-project

- cargo lambda build --release --arm64
- cargo lambda deploy --iam-role XXXXXX  --http