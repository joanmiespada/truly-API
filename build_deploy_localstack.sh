#!/bin/zsh

# example: running the command first time to create everything from scratch at localstack:
# $ ./build_deploy_localstack.sh

# example: running only terraform
# $ ./build_deploy_localstack.sh --zip_skip --keys_skip --secrets_skip --tables_skip --dns_skip

#check if awslocal and tflocal are in $PATH
awslocal --version || exit 1
tflocal --version || exit 1

#check paramaters. They allow to skip some sections
zip_skip='false'
keys_skip='false'
secrets_skip='false'
tables_skip='false'
dns_skip='false'
for arg in "$@"
do
    case $arg in
        "--zip_skip")
            zip_skip='true'
            ;;
        "--keys_skip")
            keys_skip='true'
            ;;
        "--secrets_skip")
            secrets_skip='true'
            ;;
        "--tables_skip")
            tables_skip='true'
            ;;
        "--dns_skip")
            dns_skip='true'
            ;;
    esac
done

# environment variables
export ENVIRONMENT=development
export TF_VAR_environment_flag=$ENVIRONMENT
export RUST_LOG=info
export TF_VAR_rust_log=$RUST_LOG
export TF_VAR_rust_backtrace="full"
export TF_VAR_trace_log="cargo_lambda=info"
export TF_VAR_jwt_token_time_exp_hours=8
export TF_VAR_telemetry=false
export TF_VAR_telemetry_endpoint="http://127.0.0.1:8080"
export TF_VAR_email="joanmi@espada.cat"
dns_domain="truly.test"
export TF_VAR_dns_base=$dns_domain
dns_prefix="local"
folder='target/lambda_localstack'
multi_region=("eu-central-1" "us-west-1" "ap-northeast-1")

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


echo 'running hard pre-requisits...'
declare -A mapKeys
mapKeys_string="{ "
if [[ "$keys_skip" == 'false' ]]; then

    key=$(awslocal kms create-key --multi-region --region us-east-1 --description 'cypher master key, dont use it directly. Use region replicas.' --output json --tags "TagKey=Project,TagValue=Truly" "TagKey=environment,TagValue=${ENVIRONMENT}" || exit 1)
    key_id=$(echo $key | jq -r '.KeyMetadata.KeyId')
    key_arn=$(echo $key | jq -r '.KeyMetadata.Arn')
    echo "primary key id created: ${key_arn}"
    
    for region in "${multi_region[@]}"
    do
        region_key=$(awslocal kms replicate-key --key-id $key_arn --replica-region $region  --description 'replica key, to be used only in this region assets' --output json  --tags "TagKey=Project,TagValue=Truly" "TagKey=environment,TagValue=${ENVIRONMENT}" || exit 1)
        replica_key_rpe=$(echo $region_key | jq -r '.ReplicaKeyMetadata.KeyId')
        replica_key_arn=$(echo $region_key | jq -r '.ReplicaKeyMetadata.Arn')
        echo "replica key arn created: ${replica_key_arn}"
        mapKeys[$region]=$replica_key_rpe
        mapKeys_string+="'${region}': '${replica_key_rpe}', "
    done
else
    echo 'skipping key creation, reusing current ones.'
    for region in "${multi_region[@]}"
    do
        keys=$(awslocal kms list-keys --region $region --output json | jq -r '.Keys[] | .KeyId')
        for key in "${keys[@]}"; do
            tags=$(awslocal kms list-resource-tags --key-id $key --region $region --output json)
            project=$(echo $tags | jq -r '.Tags[] | select(.TagKey=="Project" and .TagValue=="Truly")')
            if [ ! -z "$project" ]; then
                mapKeys[$region]=$key
                mapKeys_string+="'${region}': '${key}', "
                break
            fi
        done 
    done
fi
mapKeys_string+=" }"
#echo "${mapKeys_string}"

if [[ "$dns_skip" == 'false' ]]; then

    zones=$(awslocal route53 list-hosted-zones-by-name   --dns-name $dns_domain --output json | jq '[.HostedZones[]] | length' || exit 1 )

    if (( $zones <1 )); then
        echo 'creating domain zone...'
        zone_id=$(awslocal route53 create-hosted-zone --name $dns_domain --caller-reference r1 | jq -r '.HostedZone.Id' || exit 1 )
        #echo $zone_id
        for region in "${multi_region[@]}"
        do
            letters=${region%%-*}
            dns_full="${letters}-${dns_prefix}.${dns_domain}"
            echo 'creating domain by region: ' + ${dns_full}
            awslocal route53 change-resource-record-sets --hosted-zone-id $zone_id --change-batch "Changes=[{Action=CREATE,ResourceRecordSet={Name=$dns_full,Type=A,ResourceRecords=[{Value=127.0.0.1}]}}]" || exit 1
            #dig @localhost $dns_full
        done
    else
        echo 'domain has been already created'
    fi

else 
    echo 'dns creation skip'
fi


if [[ "$secrets_skip" == 'false' ]]; then
    for region in "${multi_region[@]}"
    do
        echo "secrets added at ${region}"
        cargo run -p truly_cli -- --store_secret ./truly_cli/res/secrets_development.json --create --region $region
    done
else
    echo "secrets skip"
fi

if [[ "$tables_skip" == 'false' ]]; then
    tables=$(awslocal dynamodb list-tables --region $multi_region[1] --output json | jq '[.TableNames[]] | length' )
    if (( $tables[@] <= 0 )); then
        echo "creating master tables at ${multi_region[1]}"
        cargo run -p truly_cli -- --table all --create --region $multi_region[1] || exit 1
    else
        echo "skipping master tables at ${multi_region[1]} because they already exist"
    fi

    table_names=($(awslocal dynamodb list-tables --region $multi_region[1] --output json | jq -r '.TableNames[]' ))
    for region in "${multi_region[@]:1}"
    do
        echo "deployment table replicas at ${region}"
        tables=$(awslocal dynamodb list-tables --region $region --output json | jq '[.TableNames[]] | length' )
        if (( $tables[@] <= 0 )); then
            #echo "creating replica tables..."
            for t in "${table_names[@]}"
            do
                res=$(echo "table name: ${t} source: ${multi_region[1]} replica at: ${region} "
                awslocal dynamodb update-table --table-name "${t}" --cli-input-json \
                "{
                    \"ReplicaUpdates\":
                    [
                        {
                            \"Create\": {
                                \"RegionName\": \"${region}\"
                            }
                        }
                    ]
                }" \
                --region=$multi_region[1] || exit 1)
            done
        else
            echo "tables were already replicated at ${region}"
        fi
        
    done

    echo "filling master data at ${multi_region[1]}. Note: if global tables are enabled, we can only insert only one time and it will be replicated to other tables."
    cargo run -p truly_cli -- --blockchain ./truly_cli/res/blockchain_development.json --create --region $multi_region[1] || exit 1
    cargo run -p truly_cli -- --contract  ./truly_cli/res/contract_development.json --create --region $multi_region[1] || exit 1
else
    echo "tables and master data skip"
fi

echo 'running terraform...'
cd terraform

for region in "${multi_region[@]}"
do
    letters=${region%%-*}
    region_label="localstack-${region}"
    export TF_VAR_aws_region=$region
    export TF_VAR_dns_prefix="${letters}-${dns_prefix}"
    export TF_VAR_kms_id_cypher_all_secret_keys=mapKeys[$region]
    terraform workspace new $region_label
    terraform workspace select $region_label
    echo "Planning infrastructure for ${region}..."
    tflocal plan  #-var-file="variables-localstack.tfvars"
    echo "Applying infrastructure for ${region}..."
    tflocal apply --auto-approve #-var-file="variables-localstack.tfvars" 
done

cd ..

echo 'completed!'

