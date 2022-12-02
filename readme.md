
- aws dynamodb list-tables --endpoint-url http://localhost:8000 --region=local

- NODE_ENV=development node   tableCreations.js --create
- NODE_ENV=development node   tableCreations.js --delete 
