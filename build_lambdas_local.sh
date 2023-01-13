#!/bin/bash
cargo build --release --workspace --exclude server_*

rm -rf target/lambda_local
mkdir target/lambda_local
mkdir target/lambda_local/lambda_login
cp target/release/lambda_login target/lambda_local/lambda_login/bootstrap
cd target/lambda_local/lambda_login
zip -j bootstrap.zip bootstrap 
cd ../../..

mkdir target/lambda_local/lambda_user
cp target/release/lambda_user target/lambda_local/lambda_user/bootstrap
cd target/lambda_local/lambda_user
zip -j bootstrap.zip bootstrap 
cd ../../..

mkdir target/lambda_local/lambda_admin_user
cp target/release/lambda_admin_user target/lambda_local/lambda_admin_user/bootstrap
cd target/lambda_local/lambda_admin_user
zip -j bootstrap.zip bootstrap 
cd ../../..

