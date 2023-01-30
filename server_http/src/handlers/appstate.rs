use lib_users::services::users::UsersService;
use lib_licenses::services::assets::AssetService;
use lib_config::Config;
use lib_licenses::services::owners::OwnerService;
use lib_licenses::services::nfts::NFTsService;

pub struct AppState {
    pub user_service: UsersService,
    pub app_config : Config,
    pub asset_service: AssetService,
    pub owner_service: OwnerService, 
    pub blockchain_service: NFTsService,
}