#!/bin/zsh

# example: running the command first time to create everything from scratch at localstack:
# $ ./build_deploy_stage.sh

# example: running only terraform
# $ ./build_deploy_stage.sh --zip_skip  --secrets_skip --tables_skip 

#check if aws and tf are in $PATH
aws --version || exit 1
terraform --version || exit 1
tflint --version || exit 1
git --version || exit 1
jq --version || exit 1

source scripts.sh # load functions

typeset -A build_specific_lambda
source lambdas.sh # load lambdas
lambdas_to_build=($(echo $lambdas | jq -r '.[].name'))

#check paramaters. They allow to skip some sections
images_skip='false'
secrets_skip='false'
tables_skip='false'
terraform_skip='false'
git_skip='false'
for arg in "$@"
do
    case $arg in
        "--images_skip")
            images_skip='true'
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
        "--git_skip")
            git_skip='true'
            ;;
        "--lambdas")
            shift
            lambdas_to_build=("$@")
            parse_lambda_names
            break # No need to parse further arguments as they are treated as lambda names
            ;;
    esac
done

# environment variables
export ENVIRONMENT="stage"
export TF_VAR_environment_flag=$ENVIRONMENT
export RUST_LOG="info"
#export TF_VAR_telemtry=false
#export TF_VAR_telemetry_endpoint=""
export TF_VAR_rust_log="info"
export TF_VAR_trace_level="info"
export TF_VAR_rust_backtrace="full"
export TF_VAR_trace_log="cargo_lambda=info"
export TF_VAR_jwt_token_time_exp_hours=8
export TF_VAR_telemetry=false
export TF_VAR_email="joanmi@espada.cat"
hosted_zone_id="Z07710191JBEMS0WHKVOJ" #it must be create earlier.
export TF_VAR_hosted_zone_id=$hosted_zone_id
dns_domain="truly.video"
profile="truly"
export AWS_PROFILE=$profile
export TF_VAR_dns_base=$dns_domain
dns_prefix="staging"
export TF_VAR_dns_prefix=$dns_prefix
export ses_dns_domain="mail1.${dns_domain}"
export TF_VAR_ses_domain=$ses_dns_domain
export TF_VAR_ses_from_email="joan@$ses_dns_domain"

multi_region=("eu-west-1")

typeset -A map_email_servers
map_email_servers=(
  "eu-west-1" "email-smtp.eu-west-1.amazonaws.com"
)

account_id=$(aws sts get-caller-identity --query Account --profile $profile --output text)




if [[ "$git_skip" == 'false' ]]; then
    git add .
    if git diff --staged --quiet; then
        echo "No changes to commit."
    else
        git commit -m "deploying to $ENVIRONMENT"
        git push
    fi
fi

if [[ "$images_skip" == 'false' ]]; then

    echo $lambdas | jq -c '.[]' | while read -r lambda; do
        lambda_name=$(echo $lambda | jq -r '.name')    
        imageVersion=$(echo $lambda | jq -r '.version')
        docker_path=$(echo $lambda | jq -r '.path')
        repo_name="$lambda_name-$ENVIRONMENT"

        if [[ -n "${build_specific_lambda[$lambda_name]}" || ${#build_specific_lambda} -eq 0 ]]; then


            echo "Building $lambda_name..."
            docker build --platform=linux/arm64  -t $lambda_name:$imageVersion -f $docker_path . || exit 1

            for region in "${multi_region[@]}"
            do
                reg=${region//-/_}
                eval "declare -A map_lambda_repos_${reg}"
                # Check if ECR repository exists
                if ! aws ecr describe-repositories --region $region --profile $profile --repository-names $repo_name &> /dev/null; then
                    echo "Repository $repo_name doesn't exist in $region. Creating..."
                    res=$(aws ecr create-repository --repository-name $repo_name --region $region --profile $profile || exit 1)
                fi
                repo_url="$account_id.dkr.ecr.$region.amazonaws.com/$repo_name"
                aws ecr get-login-password --region $region --profile $profile  | docker login --username AWS --password-stdin $repo_url || exit 1
                docker tag $lambda_name:$imageVersion $repo_url:$imageVersion  || exit 1
                docker push $repo_url:$imageVersion  || exit 1
                eval "map_lambda_repos_${reg}[$lambda_name]=${repo_url}:${imageVersion}"
                
            done

        else
            for region in "${multi_region[@]}"
            do
                reg=${region//-/_}
                eval "declare -A map_lambda_repos_${reg}"
                repo_url="$account_id.dkr.ecr.$region.amazonaws.com/$repo_name"
                eval "map_lambda_repos_${reg}[$lambda_name]=${repo_url}:${imageVersion}"
            done
        fi
    done
else
    echo 'skipping lambdas compilation, reusing current images already pushed'

    echo $lambdas | jq -c '.[]' | while read -r lambda; do
        lambda_name=$(echo $lambda | jq -r '.name')
        imageVersion=$(echo $lambda | jq -r '.version')
        docker_path=$(echo $lambda | jq -r '.path')
        repo_name="$lambda_name-$ENVIRONMENT"

        for region in "${multi_region[@]}"
        do
            reg=${region//-/_}
            eval "declare -A map_lambda_repos_${reg}"
            repo_url="$account_id.dkr.ecr.$region.amazonaws.com/$repo_name"
            eval "map_lambda_repos_${reg}[$lambda_name]=${repo_url}:${imageVersion}"
        done
    done
fi


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
if [[ ${#mapKeys[@]} -eq 0 ]]; then
    echo "no keys were found! check if keys exist and/or tags are corrected annotated"
    exit 1
else
    echo "Key Ids found"
fi


echo "checking dns zone..."
zones=$(aws route53 list-hosted-zones-by-name --dns-name $dns_domain --output json | jq '[.HostedZones[]] | length' || exit 1 )

if (( $zones == 0 )); then
    echo "please create the zone first."
    exit 1
else
    echo "DNS exists"
fi

if [[ "$secrets_skip" == 'false' ]]; then
    for region in "${multi_region[@]}"
    do
        #manual secrets are here, automatic secrets are in terraform.
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

    service_names=("users" "owners" "assets" "subscriptions") 

    for service in "${service_names[@]}"
    do
        echo "checking/creating master tables for $service at ${multi_region[1]}"
        cargo run -p truly_cli -- --service $service --create --region $multi_region[1] --profile $profile
    done

    for region in "${multi_region[@]:1}"
    do
        echo "deployment table replicas at ${region}"
        #tables_json=$(aws dynamodb list-tables --region $region --output json  --profile $profile | jq -r '.TableNames[]' )
        #tables_array=("${(@f)tables_json}")
        tables_json=$(aws dynamodb list-tables --region $multi_region[1] --output json  --profile $profile | jq -r '.TableNames[]' )
        filtered_result=($(filter_tables_by_tags "$tables_json"))


        for table in "${filtered_result[@]}"
        do

                    res=$(echo "table name: ${table} source: ${multi_region[1]} replica at: ${region}..."
                    aws dynamodb update-table --table-name "${table}" --cli-input-json \
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
                    --region=$multi_region[1] --profile $profile || exit 1)
                    echo "table $table has been replicated at ${region}"
        done
        
    done

    
else
    echo "tables and master data skip"
fi

if [[ -f .env-stage ]]; then
    # Extract the value of X and store it in Y
    mathchapi=$(grep -E '^MATCHAPI_ENDPOINT\s*=' .env-stage | cut -d '=' -f2- | tr -d ' "' | xargs)
    #echo $mathchapi
else
    echo ".env-stage file not found"
    exit 1
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
        export TF_VAR_matchapi_endpoint=$mathchapi
        #echo "exporting email server for ${map_email_servers[$region]}..."
        export TF_VAR_email_server=${map_email_servers[$region]}

        cmd="ecrs=(\"\${(k)map_lambda_repos_${reg}[@]}\")"
        eval "$cmd"

        for lambda_name in "${ecrs[@]}"
        do 
            eval "repo=\${map_lambda_repos_${reg}[$lambda_name]}"
            echo "exporting ecr_${lambda_name} = ${repo}..."
            export TF_VAR_ecr_${lambda_name}="$repo"
        done

        terraform workspace new $region_label
        terraform workspace select $region_label
        echo "Planning infrastructure for ${region}..."
        #tflint --recursive 
        terraform validate || exit 1
        terraform plan -out=plan.tfplan || exit 1
        echo "Applying infrastructure for ${region}..."
        terraform apply --auto-approve plan.tfplan || exit 1
        rm plan.tfplan
    done
    cd ..
else
    echo "terraform skip"
fi

echo "At stage no dns geolocation is needed."

echo 'completed!'



