#!/bin/zsh

awslocal --version || exit 1
aws --version || exit 1
qldb --version || exit 1  #install https://docs.aws.amazon.com/qldb/latest/developerguide/data-shell.html

env="stage" 
#env="production" 
#env="development" 

profile="truly"
#profile="localstack"

export ENVIRONMENT=$env
export AWS_PROFILE=$profile

if [[ $env == "stage" ]]; then
    multi_region=("eu-west-1")
    AWS_CLI="aws"
elif [[ $env == "development" ]]; then
    multi_region=("eu-central-1")
    AWS_CLI="awslocal"
else
    multi_region=("eu-central-1" "us-west-2" "ap-northeast-1") 
    AWS_CLI="aws"
fi

echo "Creating ledgers in each region. It might require several minutes."
for region in "${multi_region[@]}"; do

    ledgers=$($AWS_CLI qldb list-ledgers --region $region --output json | jq -r '.Ledgers[].Name' | wc -l)

    if (( $ledgers <= 0 )); then
        echo "Creating ledger at $region..."
        
        if [[ $env == "production" ]]; then
            create_ledger_output=$($AWS_CLI qldb create-ledger --name truly-assets-ledger --permissions-mode STANDARD --deletion-protection --region $region --tags "Projecte=Truly,Service=Api,Environment=${ENVIRONMENT}" 2>&1)
        else
            create_ledger_output=$($AWS_CLI qldb create-ledger --name truly-assets-ledger --permissions-mode ALLOW_ALL --no-deletion-protection --region $region --tags "Projecte=Truly,Service=Api,Environment=${ENVIRONMENT}" 2>&1)
        fi

        if [[ $? != 0 ]]; then
            echo "Error creating ledger at $region: $create_ledger_output"
            exit 1
        else
            echo "re run this command when ledger has been fully created to create table/indexes inside the ledger..."
        fi
    else

        ledgers=$($AWS_CLI qldb list-ledgers --region $region --output json | jq -r '.Ledgers[].Name' | wc -l)
        
        if [[ $env == "development" ]]; then
            ENDPOINT="-s http://localhost:4566"
        else
            ENDPOINT=""
        fi

        echo "Creating table and indexes at $region in the ledger... "
        qldb --ledger truly-assets-ledger --region $region -f ion --profile $profile $ENDPOINT <<< "CREATE TABLE Asset;" || exit 1
        sleep 5
        
        qldb --ledger truly-assets-ledger --region $region -f ion --profile $profile $ENDPOINT <<< "CREATE INDEX ON Asset (asset_hash);" || exit 1
        sleep 5
        
        qldb --ledger truly-assets-ledger --region $region -f ion --profile $profile $ENDPOINT  <<< "CREATE INDEX ON Asset (asset_id);" || exit 1
        sleep 5
        
        echo "tables and index created successfully";
    fi
    
done

