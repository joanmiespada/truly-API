
# Truly

Truly is an open source project aiming to identify fake videos that have been slightly modified from the original ones to switch their context. This new tool detects similar videos and uses blockchain to store a video fingerprint inside the blockchain ledger as proof of their existence. It is a REST API developed in RUST and Python, and it can work with Ethereum (solidity), SUI (move) and AWS QLDB blockchains.

## Pre-requistes

Rust toolchain update

- rustup update
- docker-compose up -d

## Localstack status

- <http://localhost:4566/health>

## Create Tables and basic data

- aws dynamodb list-tables    --endpoint-url <http://localhost:4566>
- aws dynamodb describe-table --endpoint-url <http://localhost:4566> --table-name truly_users

## Compile and run lambdas local dev

cargo build --release --workspace

Run workspace with all lambdas

- ENVIRONMENT=development cargo lambda watch
- open with postmand <http://localhost:9000/lambda-url/lambda_login>

<https://www.cargo-lambda.info/guide/getting-started.html#step-2-create-a-new-project>

- cargo lambda start
- <http://localhost:9000/lambda-url/xxxx/>...

## compile lambdas for production

- cargo lambda build --release --arm64 --output-format zip --workspace  --exclude server_* --lambda-dir target/lambda_arm64

## infrastructure deployment

Follow scripts build_deploy_xxxxx.sh

## Install telemetry

```bash
wget https://github.com/quickwit-oss/quickwit-datasource/releases/download/v0.2.0/quickwit-quickwit-datasource-0.2.0.zip \
&& mkdir -p grafana-storage/plugins \
&& unzip quickwit-quickwit-datasource-0.2.0.zip -d grafana-storage/plugins
```

## Dependencies to use localstack

```bash
brew install python
pip install awscli-local
pip install terraform-local
```
