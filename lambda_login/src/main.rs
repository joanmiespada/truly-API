
// https://blog.logrocket.com/deploy-lambda-functions-rust/

use lib_config::Config;
use lib_users::repositories::users::UsersRepo;
use lib_users::services::users::UsersService;

mod my_lambda;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::new();
    config.setup().await;

    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);

    my_lambda::lambda_main(&config, &user_service).await
}
