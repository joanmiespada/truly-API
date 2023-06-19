#!/bin/bash

export RUST_LOG=info
export AWS_REGION='eu-central-1'
export TF_VAR_aws_region=$AWS_REGION
folder='target/lambda_localstack'

echo 'compiling lambdas...'
cargo build --workspace --exclude server_* 

rm -rf $folder
mkdir $folder

lambdas=("lambda_login" "lambda_admin" "lambda_after_video" "lambda_license" "lambda_mint" "lambda_user")

echo 'zipping lambdas...'
for lambda_name in "${lambdas[@]}"
do
    mkdir ${folder}/${lambda_name}
    cp target/debug/${lambda_name} ${folder}/${lambda_name}/bootstrap
    cd ${folder}/${lambda_name}
    zip -j -q bootstrap.zip bootstrap
    cd ../../..
done 
export TF_VAR_lambda_deploy_folder=../${folder}/

echo 'running hard pre-requisits...'
key_id=$(awslocal kms create-key --output json | jq -r '.KeyMetadata.KeyId')
echo "key id created: ${key_id}"
export TF_VAR_kms_id_cypher_all_secret_keys=$key_id

dns_domain="truly.test"
export TF_VAR_dns_base=$dns_domain
dns_prefix="local"
export TF_VAR_dns_prefix=$dns_prefix
dns_full="${dns_prefix}.${dns_domain}"
echo "dns: ${dns_full}"

zones=$(awslocal route53 list-hosted-zones-by-name   --dns-name $dns_domain --output json | jq '[.HostedZones[]] | length')

if (( zones <1 )); then
    echo 'creating domain zone...'
    zone_id=$(awslocal route53 create-hosted-zone --name $dns_domain --caller-reference r1 | jq -r '.HostedZone.Id')
    #echo $zone_id
    awslocal route53 change-resource-record-sets --hosted-zone-id $zone_id --change-batch "Changes=[{Action=CREATE,ResourceRecordSet={Name=$dns_full,Type=A,ResourceRecords=[{Value=127.0.0.1}]}}]"
    dig @localhost $dns_full
else
    echo 'domain has been already created'
fi
tables=$(awslocal dynamodb list-tables --region eu-central-1 --output json | jq '[.TableNames[]] | length' )
if (( tables <1 )); then
    echo 'creating tables...'
    ENVIRONMENT=development cargo run -p truly_cli -- --table all --create
else
    echo 'tables were already created'
fi

echo 'running terraform...'
cd terraform

tflocal plan -var-file="variables-localstack.tfvars"
tflocal apply -var-file="variables-localstack.tfvars" --auto-approve

cd ..
