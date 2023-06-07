use std::{fmt, str::FromStr};

use async_trait::async_trait;
use chrono::Utc;
use lib_config::config::Config;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::nft::{
    TokenHasBeenMintedAlreadyError, TokenMintingProcessHasBeenInitiatedError, TokenNotSuccessfullyMintedPreviously,
};
use crate::models::block_tx::BlockchainTx;
use crate::blockchains::chain::NFTsRepository;
use crate::repositories::keypairs::{KeyPairRepo, KeyPairRepository};
use lib_licenses::errors::video::VideoNotYetLicensed;
use lib_licenses::models::asset::{Asset, MintingStatus, VideoLicensingStatus};
use lib_licenses::services::assets::{AssetManipulation, AssetService};
use lib_licenses::services::owners::{OwnerManipulation, OwnerService};

use super::block_tx::{BlockchainTxManipulation, BlockchainTxService};

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[async_trait]
pub trait NFTsManipulation {
    async fn prechecks_before_minting(
        &self,
        asset_id: &Uuid,
        user_id: &String,
        price: &Option<u64>,
    ) -> ResultE<Asset>;
    async fn try_mint(
        &self,
        asset_id: &Uuid,
        user_id: &String,
        price: &Option<u64>,
    ) -> ResultE<BlockchainTx>;
    async fn get(&self, asset_id: &Uuid) -> ResultE<NTFContentInfo>;
}

#[derive(Debug)]
pub struct NFTsService {
    blockchain: Box<dyn NFTsRepository + Sync + Send>,
    keys_repo: KeyPairRepo,
    asset_service: AssetService,
    owner_service: OwnerService,
    tx_service: BlockchainTxService,
    config: Config,
}

impl NFTsService {
    pub fn new(
        repo: Box<dyn NFTsRepository + Sync + Send>,
        keys_repo: KeyPairRepo,
        asset_service: AssetService,
        owner_service: OwnerService,
        tx_service: BlockchainTxService,
        config: Config,
    ) -> NFTsService {
        NFTsService {
            blockchain: repo,
            keys_repo,
            asset_service,
            owner_service,
            config,
            tx_service,
        }
    }
}

#[async_trait]
impl NFTsManipulation for NFTsService {
    async fn prechecks_before_minting(
        &self,
        asset_id: &Uuid,
        user_id: &String,
        _price: &Option<u64>,
    ) -> ResultE<Asset> {
        let asset = self.asset_service.get_by_id(asset_id).await?;
        if *asset.mint_status() == MintingStatus::CompletedSuccessfully {
            return Err(TokenHasBeenMintedAlreadyError {
                0: asset_id.to_owned(),
            }
            .into());
        }
        let last_update = asset.last_update_time();
        let diff = Utc::now() - *last_update;
        let diff_min = diff.num_minutes();
        const LIMIT: i64 = 5;

        if *asset.mint_status() == MintingStatus::Started && diff_min < LIMIT {
            return Err(TokenMintingProcessHasBeenInitiatedError {
                0: asset_id.to_owned(),
                1: LIMIT,
            }
            .into());
        }

        if *asset.video_licensing_status() != VideoLicensingStatus::AlreadyLicensed {
            return Err(VideoNotYetLicensed {}.into());
        }

        //check ownership between user and asset
        self.owner_service
            .get_by_user_asset_ids(asset_id, user_id)
            .await?;

        //TODO: check price minimum ammount!!!!!

        Ok(asset.to_owned())
    }

    #[tracing::instrument()]
    async fn try_mint(
        &self,
        asset_id: &Uuid,
        user_id: &String,
        price: &Option<u64>,
    ) -> ResultE<BlockchainTx> {
        let asset = self
            .prechecks_before_minting(asset_id, user_id, price)
            .await?;

        let user_wallet_address = self.keys_repo.get_or_create(user_id).await?;

        self.asset_service
            .mint_status(asset_id, &None, MintingStatus::Started)
            .await?;
        let counter = asset.counter().unwrap();
        let hash_file = asset.hash().to_owned().unwrap();
        let hash_algo = asset.hash_algorithm().to_owned().unwrap();
        let transaction_op = self
            .blockchain
            .add(asset_id, &user_wallet_address, &hash_file, &hash_algo , price, &counter)
            .await;

        match transaction_op {
            Err(e) => {
                let asset = self.asset_service.get_by_id(asset_id).await?;
                //it has been previously minted by other process...
                if *asset.mint_status() == MintingStatus::CompletedSuccessfully {
                    let tx = asset.minted_tx().clone().unwrap();
                    //let tx_hash = H256::from_str(tx.as_str()).unwrap();
                    let transact = self.tx_service.get_by_id(&tx).await?;
                    return Ok(transact);
                } else {
                    self.asset_service
                        .mint_status(asset_id, &None, MintingStatus::Error)
                        .await?;

                    let tx_paylaod = BlockchainTx::new(
                        asset_id.clone(),
                        Utc::now(),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        self.blockchain.contract_id(),
                        Some(e.to_string()),
                    );
                    //tx_paylaod.set_asset_id(asset_id);
                    //tx_paylaod.set_result(&e.to_string());

                    self.tx_service.add(&tx_paylaod).await?;
                    return Err(e.into());
                }
            }
            Ok(transaction) => {
                // let tx_res =  match *transaction.tx() {
                //     None => None,
                //     Some(x)=>{ Some( H256::to_string(&x)  )}
                // };
                let tx_res = transaction.tx().clone();
                self.asset_service
                    .mint_status(
                        asset_id,
                        //transaction.tx(),
                        &tx_res,
                        MintingStatus::CompletedSuccessfully,
                    )
                    .await?;

                self.tx_service.add(&transaction).await?;
                Ok(transaction)
            }
        }
    }

    #[tracing::instrument()]
    async fn get(&self, asset_id: &Uuid) -> ResultE<NTFContentInfo> {

        let txs = self.tx_service.get_by_asset_id(asset_id).await?;

        let successfully = txs.into_iter().filter(|x| x.tx_error().as_deref() == None ).collect::<Vec<BlockchainTx>>(); 
        let successfully =  successfully.first();

        if successfully == None {
            return Err( TokenNotSuccessfullyMintedPreviously{ 0: asset_id.clone()}.into());
        }
        let successfully = successfully.unwrap();
        let token = successfully.tx().clone().unwrap();

        let aux = self.blockchain.get(&token).await?;
        let state;
        if let Some(sts) = aux.state {
            state = NTFState::from_str(&sts.to_string()).unwrap()
        }else{
            state = NTFState::Active;
        }
        let res = NTFContentInfo {
            hash_file: aux.hashFile,
            hash_algorithm: aux.hashAlgo,
            uri: aux.uri,
            price: aux.price,
            state,
        };
        Ok(res)
    }
}

impl Clone for NFTsService {
    #[tracing::instrument()]
    fn clone(&self) -> NFTsService {
        let aux = NFTsService {
            blockchain: self.blockchain.clone(),
            keys_repo: self.keys_repo.clone(),
            owner_service: self.owner_service.clone(),
            asset_service: self.asset_service.clone(),
            config: self.config.clone(),
            tx_service: self.tx_service.clone(),
        };
        return aux;
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NTFContentInfo {
    pub hash_file: String,
    pub hash_algorithm: String,
    pub uri: Option<String>,
    pub price: Option<u64>,
    pub state: NTFState,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NTFState {
    Active,
    Inactive,
}
impl fmt::Debug for NTFState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Inactive => write!(f, "Inactive"),
        }
    }
}
impl fmt::Display for NTFState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Inactive => write!(f, "Inactive"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseNTFStateError;
impl std::str::FromStr for NTFState {
    type Err = ParseNTFStateError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "Active" => Ok(NTFState::Active),
            "Inactive" => Ok(NTFState::Inactive),
            _ => Err(ParseNTFStateError),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateNFTAsync {
    pub price: u64,
    pub asset_id: Uuid,
    pub user_id: String,
    pub tries: usize,
}

impl CreateNFTAsync {
    pub fn new(user_id: &String, asset_id: &Uuid, price: u64) -> CreateNFTAsync {
        CreateNFTAsync {
            price,
            asset_id: asset_id.to_owned(),
            user_id: user_id.to_owned(),
            tries: 0,
        }
    }
    pub fn increase_try(&mut self) {
        self.tries += 1;
    }

    pub fn get_tries(&self) -> usize {
        self.tries
    }
}

impl std::fmt::Display for CreateNFTAsync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "user_id: {} asset_id: {} price: {}",
            self.user_id,
            self.asset_id.to_string(),
            self.price
        )
    }
}
