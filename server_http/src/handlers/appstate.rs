use lib_users::services::users::{UsersService};
use lib_licenses::services::assets::{AssetService};
use lib_config::{Config};

pub struct AppState {
    pub user_service: UsersService,
    pub app_config : Config,
    pub asset_service: AssetService
}