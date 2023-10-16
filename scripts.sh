#!/bin/zsh


table_exists() {
        local table_to_check="$1"
        shift
        local -a existing_tables=("$@")

        for existing_table in "${existing_tables[@]}"; do
            if [[ "$table_to_check" == "$existing_table" ]]; then
                return 0
            fi
        done
        return 1
    }

filter_tables_by_tags() {
    local -a tables_to_filter=("${(@f)$(echo "$1")}")  # We use zsh-specific parameter expansion flags here

    local filtered_tables=()

    for table in "${tables_to_filter[@]}"; do
        
        # Retrieve the table's ARN
        local table_arn=$(aws dynamodb describe-table --table-name "${table}" --region $region --profile $profile --output json | jq -r '.Table.TableArn')
        #echo $table_arn 
        # If the table doesn't exist or there was an error, continue to the next iteration
        if [ -z "$table_arn" ]; then
            #echo "Error retrieving ARN for table: $table"
            continue
        fi
        
        # Get tags for the table using its ARN
        local tags_json=$(aws dynamodb list-tags-of-resource --resource-arn "$table_arn" --region $region --profile $profile)
        
        # Check if the table has the specified tags using the predefined variables
        local has_project_tag=$(echo $tags_json | jq --arg PROJECT "$PROJECT" 'select(.Tags[] | select(.Key == "PROJECT" and .Value == $PROJECT))')
        local has_service_tag=$(echo $tags_json | jq --arg SERVICE "$SERVICE" 'select(.Tags[] | select(.Key == "SERVICE" and .Value == $SERVICE))')
        local has_environment_tag=$(echo $tags_json | jq --arg ENVIRONMENT "$ENVIRONMENT" 'select(.Tags[] | select(.Key == "ENVIRONMENT" and .Value == $ENVIRONMENT))')

        if [[ -n $has_project_tag && -n $has_service_tag && -n $has_environment_tag ]]; then
            filtered_tables+=("$table")
        fi
    done

    echo "${filtered_tables[@]}"
}
