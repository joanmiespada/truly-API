#!/bin/zsh

env="stage" 
#env="production" 
export ENVIRONMENT=$env
profile="truly"
export AWS_PROFILE=$profile

if [[ $env == "stage" ]]; then
    multi_region=("eu-west-1")
else
    multi_region=("eu-central-1" "us-west-2" "ap-northeast-1") 
fi

echo "Creating ledgers in each region. It might require several minutes."
for region in "${multi_region[@]}"; do
    ledgers=$(aws qldb list-ledgers --region $region --output json | jq -r '.Ledgers[].Name' | wc -l)
    if (( $ledgers <= 0 )); then
        echo "Creating ledger at $region..."
        create_ledger_output=$(aws qldb create-ledger --name truly-assets-ledger --permissions-mode STANDARD --region $region --tags "Projecte=Truly,Service=Api,Environment=${ENVIRONMENT}" 2>&1)
        if [[ $? != 0 ]]; then
            echo "Error creating ledger at $region: $create_ledger_output"
            exit 1
        else
            echo "re run this command when ledger has been fully created to create table/indexes..."
        fi
    else
        echo "Creating table and indexes at $region in the ledger... "
        qldb --ledger truly-assets-ledger --region $region -f ion --profile $profile >/dev/null <<< "CREATE TABLE Asset;" || exit 1
        sleep 5s
        qldb --ledger truly-assets-ledger --region $region -f ion --profile $profile >/dev/null <<< "CREATE INDEX ON Asset (asset_hash);" || exit 1
        sleep 5s
        qldb --ledger truly-assets-ledger --region $region -f ion --profile $profile >/dev/null <<< "CREATE INDEX ON Asset (asset_id);" || exit 1
        echo "tables and index created successfully"
    fi
    
done

