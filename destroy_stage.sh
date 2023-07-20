#!/bin/zsh

# example: running the command first time to create everything from scratch at localstack:
# $ ./build_deploy_stage.sh

# example: running only terraform
# $ ./build_deploy_stage.sh --zip_skip  --secrets_skip --tables_skip 

#check if aws and tf are in $PATH
aws --version || exit 1
terraform --version || exit 1

#check paramaters. They allow to skip some sections
zip_skip='false'
secrets_skip='false'
tables_skip='false'
terraform_skip='false'
for arg in "$@"
do
    case $arg in
        "--zip_skip")
            zip_skip='true'
            ;;
        "--tables_skip")
            tables_skip='true'
            ;;
        "--terraform_skip")
            terraform_skip='true'
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
multi_region=("eu-west-1")


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

echo "At stage no dns geolocation is needed. No rules to be destroyed"

if [[ "$terraform_skip" == 'false' ]]; then
    echo 'running terraform...'
    cd terraform
    for region in "${multi_region[@]}"
    do
        letters=${region%%-*}
        region_label="$ENVIRONMENT-${region}"
        export TF_VAR_aws_region=$region
        export TF_VAR_dns_prefix="${letters}-${dns_prefix}"
        export TF_VAR_kms_id_cypher_all_secret_keys=mapKeys[$region]
        terraform workspace select $region_label
        echo "Planning infrastructure for ${region}..."
        terraform plan
        terraform destroy --auto-approve || exit 1
    done
    cd ..
else
    echo "skip terraform destroy"
fi


if [[ "$tables_skip" == 'false' ]]; then

    table_names=($(aws dynamodb list-tables  --region $multi_region[1] --output json | jq -r '.TableNames[]' ))
    for region in "${multi_region[@]}"
    do
        echo "deleting table replicas at ${region}"
        tables=$(aws dynamodb list-tables  --region $region --output json | jq '[.TableNames[]] | length' )
        if (( $tables[@] > 0 )); then
            #echo "creating replica tables..."
            for t in "${table_names[@]}"
            do
                echo "deleting ${t} at ${region}..."
                res=$(aws dynamodb delete-table --table-name "${t}" --region $region --output json || exit 1)
            done
        else
            echo "no tables at ${region}"
        fi
        
    done

else
    echo "tables and master data skip"
fi

echo "no secrests deleted, you would need to remove them manually."

echo "no keys deleted, you would need to remove them manually."

echo "no DNS zone deleted, you would need to remove them manually."

echo "no Ledger deleted, you would need to remove them manually."

echo 'completed!'

