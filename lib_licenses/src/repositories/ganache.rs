use lib_config::Config;
use uuid::Uuid;
use web3::{Web3, transports::Http};


type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error>>;

//#[async_trait]
pub trait NFTsRepository {
    //async 
    fn add(&self, asset_id: &Uuid, user_id: &String) -> ResultE<()>;
}

#[derive(Clone, Debug)]
pub struct GanacheRepo {
    web3: Web3<Http>
}

impl GanacheRepo {
    pub fn new(conf: &Config) -> GanacheRepo {
        let transport = web3::transports::Http::new( conf.env_vars().blockchain_url() ).unwrap();
        let gateway =web3::Web3::new(transport); 
        GanacheRepo {
            web3 : gateway
        }
    }

}

//#[async_trait]
impl NFTsRepository for GanacheRepo {
    
    //async 
    fn add(&self, _asset_id: &Uuid, _user_id: &String) -> ResultE<()>{
        Ok(())

    }
}