#cargo build --release --workspace --exclude server_*
rm -rf target/lambda_local
mkdir target/lambda_local
mkdir target/lambda_local/lambda_login
cp target/release/lambda_login target/lambda_local/lambda_login/bootstrap
cd target/lambda_local/lambda_login
zip bootstrap.zip bootstrap 
