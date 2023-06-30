#!/bin/zsh

#check if awslocal is in $PATH
awslocal --version || exit 1
tflocal --version || exit 1

#check if we want to reuse current zip files or not. With --zip option it will reuse current folders
zip_skip='false'
for arg in "$@"
do
    if [ "$arg" = "--zip_skip" ] ; then
        zip_skip='true'
        break
    fi
done

export ENVIRONMENT=development
export RUST_LOG=info
folder='target/lambda_localstack'

if [[ "$zip_skip" == 'false' ]]; then
    
    echo 'compiling lambdas...'
    cargo build --workspace --exclude server_*
    
    if [ $? -ne 0 ]; then
        echo 'compiling error, please check cargo build.'
        exit 1
    fi
    
    
    rm -rf $folder
    mkdir $folder
    
    lambdas=("lambda_login" "lambda_admin" "lambda_after_video" "lambda_license" "lambda_mint" "lambda_user")
    
    echo 'zipping lambdas...'
    for lambda_name in "${lambdas[@]}"
    do
        echo $lambda_name
        mkdir ${folder}/${lambda_name}
        cp target/debug/${lambda_name} ${folder}/${lambda_name}/bootstrap
        cd ${folder}/${lambda_name}
        zip -j -q bootstrap.zip bootstrap
        if [ $? -ne 0 ]; then
            echo `zipping bootstrap file error at ${lambda_name}, please check zip command and/or folders.`
            exit 1
        fi
        cd ../../..
    done
else
    echo 'skipping lambdas compilation and zip, reusing current folders and zip files.'
fi
export TF_VAR_lambda_deploy_folder=../${folder}
echo "lambdas will be seek at: ${TF_VAR_lambda_deploy_folder}"

multi_region=("eu-central-1" "us-west-1")
#multi_region=("eu-central-1")

echo 'running hard pre-requisits...'

key=$(awslocal kms create-key --multi-region --region us-east-1 --description 'cypher master key, dont use it directly. Use region replicas.' --output json --tags "TagKey=Project,TagValue=Truly" "TagKey=environment,TagValue=${ENVIRONMENT}" || exit 1)
key_id=$(echo $key | jq -r '.KeyMetadata.KeyId')
key_arn=$(echo $key | jq -r '.KeyMetadata.Arn')
echo "primary key id created: ${key_arn}"
declare -A mapKeys
mapKeys_string="{ "
for region in "${multi_region[@]}"
do
    region_key=$(awslocal kms replicate-key --key-id $key_arn --replica-region $region  --description 'replica key, to be used only in this region assets' --output json  --tags "TagKey=Project,TagValue=Truly" "TagKey=environment,TagValue=${ENVIRONMENT}" || exit 1)
    replica_key_rpe=$(echo $region_key | jq -r '.ReplicaKeyMetadata.KeyId')
    replica_key_arn=$(echo $region_key | jq -r '.ReplicaKeyMetadata.Arn')
    echo "replica key arn created: ${replica_key_arn}"
    mapKeys[$region]=$replica_key_rpe
    mapKeys_string+="'${region}': '${replica_key_rpe}', "
done
mapKeys_string+=" }"
export TF_VAR_kms_id_cypher_all_secret_keys=$mapKeys_string
echo "${mapKeys_string}"

dns_domain="truly.test"
export TF_VAR_dns_base=$dns_domain
dns_prefix="local"
export TF_VAR_dns_prefix=$dns_prefix
dns_full="${dns_prefix}.${dns_domain}"
echo "dns: ${dns_full}"

zones=$(awslocal route53 list-hosted-zones-by-name   --dns-name $dns_domain --output json | jq '[.HostedZones[]] | length')

if (( $zones <1 )); then
    echo 'creating domain zone...'
    zone_id=$(awslocal route53 create-hosted-zone --name $dns_domain --caller-reference r1 | jq -r '.HostedZone.Id')
    #echo $zone_id
    awslocal route53 change-resource-record-sets --hosted-zone-id $zone_id --change-batch "Changes=[{Action=CREATE,ResourceRecordSet={Name=$dns_full,Type=A,ResourceRecords=[{Value=127.0.0.1}]}}]"
    dig @localhost $dns_full
else
    echo 'domain has been already created'
fi

for region in "${multi_region[@]}"
do
    echo "deployment at ${region}"
    echo "tables..."
    tables=$(awslocal dynamodb list-tables --region $region --output json | jq '[.TableNames[]] | length' )
    if (( tables <1 )); then
        echo "creating tables..."
        cargo run -p truly_cli -- --table all --create --region $region
    else
        echo "tables were already created at ${region}"
    fi
    
    echo "secrets manager values..."
    cargo run -p truly_cli -- --store_secret ./truly_cli/res/secrets_development.json --create --region $region

done

for region in "${multi_region[@]}"
do
    echo "filling master data at ${region}. Note: if global tables are enabled, we can only insert only one time and it will be replicated to other tables."
    #cargo run -p truly_cli -- --blockchain ./truly_cli/res/blockchain_development.json --create --region $region || exit 1
    #cargo run -p truly_cli -- --contract  ./truly_cli/res/contract_development.json --create --region $region || exit 1
done

# echo 'running terraform...'
# cd terraform

# for region in "${multi_region[@]}"
# do
#     region_label="localstack-${region}"
#     export TF_VAR_aws_region=$region
#     export TF_VAR_aws_region=$region
#     terraform workspace new $region_label
#     terraform workspace select $region_label
#     echo "Planning infrastructure for ${region}..."
#     tflocal plan -var-file="variables-localstack.tfvars"
#     echo "Applying infrastructure for ${region}..."
#     tflocal apply -var-file="variables-localstack.tfvars" --auto-approve
# done

cd ..

echo 'completed!'

# function join_by_comma() {
#     local IFS=","
#     echo "$*"
# }