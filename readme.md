
- aws dynamodb list-tables --endpoint-url http://localhost:8000 --region=local

- NODE_ENV=development node  ./scripts/tableCreations.js --create
- NODE_ENV=development node  ./scripts/tableCreations.js --delete 
- NODE_ENV=development node  ./scripts/admin_user.js #create admin user

- cargo build
- cargo run

https://www.cargo-lambda.info/guide/getting-started.html#step-2-create-a-new-project

- cargo lambda build --release --arm64
- cargo lambda deploy --iam-role XXXXXX  --http