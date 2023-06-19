aws_region = "eu-central-1"
#create local key: awslocal kms create-key or with truly_cli or use the script: build_deploy_localstack.sh
kms_id_cypher_all_secret_keys = "" #"cae38c03-e275-4355-ac4c-b4f04f731da5"
environment_flag = "stage"
rust_backtrace = "full"
trace_log="cargo_lambda=info"
jwt_token_time_exp_hours=8
dns_prefix="local"