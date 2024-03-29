#!/bin/zsh

# example: running the command first time to create everything from scratch at localstack:
# $ ./build_deploy_production.sh

# example: running only terraform
# $ ./build_deploy_production.sh --zip_skip  --secrets_skip --tables_skip --geoloc_skip

#check if aws and tf are in $PATH
aws --version || exit 1
terraform --version || exit 1
qldb --version || exit 1  #install https://docs.aws.amazon.com/qldb/latest/developerguide/data-shell.html

#check paramaters. They allow to skip some sections
zip_skip='false'
secrets_skip='false'
tables_skip='false'
terraform_skip='false'
geoloc_skip='false'
ledger_skip='false'
for arg in "$@"
do
    case $arg in
        "--zip_skip")
            zip_skip='true'
            ;;
        "--secrets_skip")
            secrets_skip='true'
            ;;
        "--tables_skip")
            tables_skip='true'
            ;;
        "--terraform_skip")
            terraform_skip='true'
            ;;
        "--geoloc_skip")
            geoloc_skip='true'
            ;;
        "--ledger_skip")
            ledger_skip='true'
            ;;
    esac
done

# environment variables
export ENVIRONMENT=stage
export TF_VAR_environment_flag=$ENVIRONMENT
export RUST_LOG=info
export TF_VAR_rust_log=$RUST_LOG
export TF_VAR_rust_backtrace="full"
export TF_VAR_trace_log="cargo_lambda=info"
export TF_VAR_jwt_token_time_exp_hours=8
export TF_VAR_telemetry=false
export TF_VAR_telemetry_endpoint="http://127.0.0.1:8080"
export TF_VAR_email="joanmi@espada.cat"
dns_domain="truly.video"
profile="truly"
export AWS_PROFILE=$profile
export TF_VAR_dns_base=$dns_domain
dns_prefix="stage"
architecture="aarch64-linux-gnu"
#path_base='/Users/joanmiquelespadasabat/Projects/tron/API/cross-compile/openssl/'${architecture}
path_base=$(pwd)'/cross-compile/openssl/'${architecture}
folder="target/lambda_${architecture}"

multi_region=("eu-central-1" "us-west-2" "ap-northeast-1") #in which regions we want to deploy or infra. First one in this list is the master.
declare -A mapGeoLocations
mapGeoLocations=(
  [us]="NA SA"
  [eu]="EU AF"
  [ap]="AS OC AU NZ"
) 


if [[ "$zip_skip" == 'false' ]]; then
    
    echo "compiling lambdas ${architecture}..."
    export OPENSSL_LIB_DIR=${path_base}/lib
    export OPENSSL_INCLUDE_DIR=${path_base}/include

    cargo lambda build --release --arm64 --output-format zip --workspace  --exclude truly_cli --lambda-dir $folder
    
    if [ $? -ne 0 ]; then
        echo 'compiling error, please check cargo build.'
        exit 1
    fi
else
    echo 'skipping lambdas compilation, reusing current folders and zip files.'
fi
export TF_VAR_lambda_deploy_folder=../${folder}
echo "lambdas will be seek at: ${TF_VAR_lambda_deploy_folder}"

echo 'running hard pre-requisits: keys and secrets'
declare -A mapKeys
mapKeys_string="{ "
echo 'searching keys already created...'
for region in "${multi_region[@]}"; do
    keys=($(aws kms list-keys --region $region --output json | jq -r '.Keys[] | .KeyId'))
    for key in "${keys[@]}"; do
        project=$(aws kms list-resource-tags --key-id $key --region $region --output json | jq -r --arg env "$ENVIRONMENT" 'select(.Tags[] | select(.TagKey=="Project" and .TagValue=="Truly")) | select(.Tags[] | select(.TagKey=="Environment" and .TagValue==$env))')
        if [[ ! -z "$project" ]]; then
            length=$(echo $project | jq 'length')
            if [[ $length -gt 0 ]]; then
                mapKeys[$region]=$key
                mapKeys_string+="'${region}': '${key}', "
                break
            fi
        fi
        
    done 
done
mapKeys_string+=" }"
#echo "${mapKeys_string}"
if [[ ${#mapKeys[@]} -eq 0 ]]; then
    echo "no keys were found! check if keys exist and/or tags are corrected annotated"
    exit 1
else
    echo "Key Ids found"
fi


echo "checking dns zone..."
zones=$(aws route53 list-hosted-zones-by-name --dns-name $dns_domain --output json | jq '[.HostedZones[]] | length' || exit 1 )

if (( $zones == 0 )); then
    echo "please create the zone first at AWS console."
    exit 1
else
    echo "DNS exists"
fi

if [[ "$secrets_skip" == 'false' ]]; then
    for region in "${multi_region[@]}"
    do
        cargo run -p truly_cli -- --store_secret ./truly_cli/res/secrets_prod_stage.json --create --region $region --profile $profile # || exit 1

        if [ $? -eq 0 ]; then
            echo "secrets added at ${region}"
        else
            echo "secret creation could be failed at ${region}, please, check"
        fi

    done
else
    echo "secrets skip, they need to be already created"
fi


if [[ "$tables_skip" == 'false' ]]; then
    tables=$(aws dynamodb list-tables --region $multi_region[1] --output json | jq '[.TableNames[]] | length' )
    if (( $tables[@] <= 0 )); then
        echo "creating master tables at ${multi_region[1]}"
        cargo run -p truly_cli -- --table all --create --region $multi_region[1] --profile $profile || exit 1
    else
        echo "skipping master tables at ${multi_region[1]} because they already exist"
    fi

    table_names=($(aws dynamodb list-tables  --region $multi_region[1] --output json | jq -r '.TableNames[]' ))
    for region in "${multi_region[@]:1}"
    do
        echo "deployment table replicas at ${region}"
        tables=$(aws dynamodb list-tables  --region $region --output json | jq '[.TableNames[]] | length' )
        if (( $tables[@] <= 0 )); then
            #echo "creating replica tables..."
            for t in "${table_names[@]}"
            do
                res=$(echo "table name: ${t} source: ${multi_region[1]} replica at: ${region} "
                aws dynamodb update-table  --table-name "${t}" --cli-input-json \
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

    #echo "filling master data at ${multi_region[1]}. Note: if global tables are enabled, we can only insert only one time and it will be replicated to other tables automatically."
    #cargo run -p truly_cli -- --blockchain ./truly_cli/res/blockchain_stage.json --create --region $multi_region[1] --profile $profile || exit 1
    #cargo run -p truly_cli -- --contract  ./truly_cli/res/contract_stage.json --create --region $multi_region[1] --profile $profile || exit 1
else
    echo "tables and master data skip"
fi

if [[ "$terraform_skip" == 'false' ]]; then
    echo 'running terraform...'
    cd terraform

    for region in "${multi_region[@]}"
    do
        letters=${region%%-*}
        region_label="$ENVIRONMENT-${region}"
        export TF_VAR_aws_region=$region
        export TF_VAR_dns_prefix="${letters}-${dns_prefix}"
        export TF_VAR_kms_id_cypher_all_secret_keys=$mapKeys[$region]
        terraform workspace new $region_label
        terraform workspace select $region_label
        echo "Planning infrastructure for ${region}..."
        terraform plan
        terraform apply --auto-approve
    done
    cd ..
    echo 'Terraform done!'
else
    echo 'Terraform skip!'
fi

if [[ "$geoloc_skip" == 'false' ]]; then
    # adding route53 geolocal balancer among regions

    output=$(awslocal route53 list-hosted-zones-by-name --dns-name $dns_domain --output json)
    zone_id=$(echo $output | jq -r '.HostedZones[0].Id' | cut -d '/' -f 3)

    echo "Enabling georouting to ${zone_id}..."
    for key in ${(k)mapGeoLocations[@]}; do
    locations=("${(@s/ /)mapGeoLocations[$key]}")
    for loc in "${locations[@]}"; do
        recordSetName="$dns_prefix.$dns_domain"
        existingRecords=$(awslocal route53 list-resource-record-sets --hosted-zone-id $zone_id)
        
        if echo "$existingRecords" | jq -e --arg recordSetName "$recordSetName." --arg loc "$loc" '.ResourceRecordSets[] | select(.Name==$recordSetName and .GeoLocation.ContinentCode==$loc)' > /dev/null; then
        echo "Skipping existing record: $key / $recordSetName, Geolocation: $loc"
        else
        echo "Adding a new GeoLocation to $key / $recordSetName at $loc"
        awslocal route53 change-resource-record-sets --hosted-zone-id $zone_id --change-batch "{ \
        \"Changes\": [ \
            { \
                \"Action\": \"CREATE\", \
                \"ResourceRecordSet\": { \
                \"Name\": \"$dns_prefix.$dns_domain\", \
                \"Type\": \"CNAME\", \
                \"TTL\": 300, \
                \"ResourceRecords\": [ \
                    { \
                    \"Value\": \"$key-$dns_prefix.$dns_domain\" \
                    } \
                ], \
                \"GeoLocation\": { \
                    \"ContinentCode\": \"$loc\" \
                } \
                } \
            } \
        ] \
        }"
        fi
    done
    done
    echo "dns geolocation applied!"
else
    echo "dns geolocation skip!"
fi

if [[ "$ledger_skip" == 'false' ]]; then
    echo "Ledger in production must be created manually in each region."
else
    echo "Ledger creation skip"
fi

echo 'completed!'



