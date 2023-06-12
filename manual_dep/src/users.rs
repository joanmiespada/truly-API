use lib_config::config::Config;
use lib_users::{
    repositories::users::UsersRepo,
    services::users::{UserManipulation, UsersService},
};

pub async fn manage_user(
    id: String,
    _create: bool,
    delete: bool,
    _environment: String,
    config: &mut Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    config.load_secrets().await;
    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);
    if delete {
        let op = user_service.remove_by_id(&id).await;
        match op {
            Err(e) => {
                println!("{}", e);
            }
            Ok(_) => {
                println!("user {} deleted!", id)
            }
        }
    } else {
        panic!("Not implemented yet")
    }

    Ok(())
}
