use lib_blockchain::services::nfts::NFTsService;
use lib_config::config::Config;
use lib_licenses::services::assets::AssetService;
use lib_licenses::services::owners::OwnerService;
use lib_users::services::users::UsersService;

pub struct AppState {
    pub user_service: UsersService,
    pub app_config: Config,
    pub asset_service: AssetService,
    pub owner_service: OwnerService,
    pub blockchain_service: NFTsService,
}
