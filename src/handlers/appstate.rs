use crate::users;
use crate::config;

pub struct AppState {
    pub user_service: users::services::users::UsersService,
    pub app_config : config::Config,
}