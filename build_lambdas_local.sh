#!/bin/bash
rm -rf target/lambda_local

cargo build --workspace --exclude server_* --exclude command_* --exclude truly_cli

mkdir target/lambda_local

lambdas=("lambda_login" "lambda_admin" "lambda_after_video" "lambda_license" "lambda_mint" "lambda_user")

#lambda_name="lambda_login"

for lambda_name in "${lambdas[@]}"
do
    mkdir target/lambda_local/${lambda_name}
    cp target/release/${lambda_name} target/lambda_local/${lambda_name}/bootstrap
    cd target/lambda_local/${lambda_name}
    zip -j -q bootstrap.zip bootstrap
    
    output=$(aws lambda create-function \
        --endpoint-url http://localhost:4566 \
        --function-name ${lambda_name} \
        --runtime provided \
        --zip-file fileb://bootstrap.zip \
        --handler function_handler \
        --role arn:aws:iam::000000000000:role/lambda-role \
        --profile localstack \
        --region eu-central-1 \
    --output json)
    
    output=$(aws lambda create-function-url-config \
        --endpoint-url http://localhost:4566 \
        --function-name ${lambda_name} \
        --auth-type NONE \
        --profile localstack \
        --region eu-central-1 \
    --output json)
    url_lambda=$(echo $output | jq -r '.FunctionUrl')
    
    echo ${lambda_name}: ${url_lambda}
    cd ../../..
done



