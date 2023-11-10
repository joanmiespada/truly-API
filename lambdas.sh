#!/bin/zsh

#FILE="lambdas.json"
lambdas=$(cat "$FILE")

parse_lambda_names() {
    for lambda in "${lambdas_to_build[@]}"; do
        build_specific_lambda[$lambda]=1
    done
}

update_lambda_versions() {
    lambda_name=$1

    # Function to increment version
    increment_version() {
        version=$1
        major=$(echo $version | cut -d '.' -f 1)
        minor=$(echo $version | cut -d '.' -f 2)
        patch=$(echo $version | cut -d '.' -f 3)
        new_patch=$((patch + 1))
        echo "$major.$minor.$new_patch"
    }

    # Function to update a specific lambda
    update_lambda() {
        name=$1
        current_version=$(jq -r --arg NAME "$name" '.[] | select(.name == $NAME) | .version' "$FILE")
        new_version=$(increment_version $current_version)
        jq --arg NAME "$name" --arg VERSION "$new_version" '(.[] | select(.name == $NAME) | .version) |= $VERSION' "$FILE" > tmp.json && mv tmp.json "$FILE"
        echo "Updated $name to version $new_version"
    }

    # Update only the specified lambda
    if [[ -n "$lambda_name" ]]; then
        echo "Updating lambda version: $lambda_name"
        update_lambda "$lambda_name"
    else
        # This part will update all lambdas (can be used when no specific lambda is specified)
        for lambda in $(jq -r '.[].name' "$FILE"); do
            echo "Updating lambda version: $lambda"
            update_lambda "$lambda"
        done
    fi
}

