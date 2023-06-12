use aws_sdk_dynamodb::types::error::ResourceNotFoundException;
use lib_config::config::Config;
use lib_users::{repositories::users::UsersRepo, services::users::{UsersService, UserManipulation, PromoteUser}, models::user::User};

pub async fn create_admin_user(
    email: String,
    password:Option<String>,
    create: bool,
    delete: bool,
    environment: String,
    config: &mut Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let er = ResourceNotFoundException::builder().build();
    config.load_secrets().await;
    let user_repo = UsersRepo::new(&config);
    let user_service = UsersService::new(user_repo);
    if create {
        let mut user = User::new();
        user.set_email(&email);
        let device = uuid::Uuid::new_v4().to_string();
        user.set_device(&device);

        let user_id = user_service.add(&mut user, &password).await?;
        user_service
            .promote_user_to(&user_id, &PromoteUser::Upgrade)
            .await?;
        println!("admin user id:{} with device: {} created.", user_id, device);
    } else {
        println!("Not implemented yet")
    }

    Ok(())
}
