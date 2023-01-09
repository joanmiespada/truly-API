use std::env;
use lib_users::services::users::UsersService;
use lib_users::services::login::{LoginOps};
use lib_users::repositories::users::UsersRepo;
use lib_users::errors::users::{UserNoExistsError};
use lib_config::Config;



#[tokio::test]
async fn it_login_user() {

    env::set_var("ENVIRONMENT", "development");
    let mut config = Config::new();
    config.setup().await;

    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);

    let device = &Some("1234".to_string());
    let email = &Some("1234".to_string());
    let passw = &Some("1234".to_string());

    let res = user_service.login(device, email, passw).await;

    let e = res.err().unwrap();


    assert!(e.is::<UserNoExistsError>())



}