use std::str::FromStr;
use crate::models::asset::VideoLicensingStatus;
use crate::models::asset::{Asset, AssetStatus, MintingStatus};
use crate::models::video::VideoResult;
use crate::repositories::assets::{AssetRepo, AssetRepository};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use validator::Validate;

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[async_trait]
pub trait AssetManipulation {
    async fn get_all(&self, page_number: u32, page_size: u32) -> ResultE<Vec<Asset>>;
    async fn get_by_id(&self, asset_id: &Uuid) -> ResultE<Asset>;
    async fn get_by_user_id(&self, user_id: &String) -> ResultE<Vec<Asset>>;
    async fn get_by_user_asset_id(&self, asset_id: &Uuid, user_id: &String) -> ResultE<Asset>;
    async fn add(&self, creation_asset: &CreatableFildsAsset, user_id: &String) -> ResultE<Uuid>;
    async fn update(&self, asset_id: &Uuid, asset: &UpdatableFildsAsset) -> ResultE<()>;
    async fn mint_status(
        &self,
        id: &Uuid,
        transaction: &Option<String>,
        sts: MintingStatus,
    ) -> ResultE<()>;
    async fn store_video_process(&self, video_res: &VideoResult) -> ResultE<()>;
}

#[derive(Debug)]
pub struct AssetService {
    repository: AssetRepo,
    //owner_service: OwnerService,
}

impl AssetService {
    pub fn new(repo: AssetRepo) -> AssetService {
        //,owner_service: OwnerService
        AssetService { repository: repo } // owner_service: owner_service.clone() }
    }
}

#[derive(Debug, Validate)]
pub struct UpdatableFildsAsset {
    #[validate(length(max = 100))]
    pub license: Option<String>,
    #[validate(length(max = 10))]
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct CreatableFildsAsset {
    #[validate(length(max = 100))]
    pub license: String,
    pub url: String,
    #[validate(length(max = 100))]
    pub hash: String,

    pub latitude: Option<f64>,
    pub longitude: Option<f64>,

    pub father: Option<Uuid>
}

#[async_trait]
impl AssetManipulation for AssetService {
    #[tracing::instrument()]
    async fn get_all(&self, page_number: u32, page_size: u32) -> ResultE<Vec<Asset>> {
        let res = self.repository.get_all(page_number, page_size).await?;
        Ok(res)
    }

    #[tracing::instrument()]
    async fn get_by_id(
        &self,
        id: &Uuid,
    ) -> std::result::Result<Asset, Box<dyn std::error::Error + Sync + Send>> {
        let res = self.repository.get_by_id(id).await?;
        Ok(res)
    }

    #[tracing::instrument()]
    async fn add(&self, creation_asset: &CreatableFildsAsset, user_id: &String) -> ResultE<Uuid> {
        creation_asset.validate()?;

        let mut asset = Asset::new();
        asset.set_state(&AssetStatus::Enabled);
        asset.set_id(&Uuid::new_v4());
        let aux = creation_asset.url.clone();
        asset.set_url(&Some(url::Url::parse(aux.as_str())?));
        asset.set_hash(&Some(creation_asset.hash.clone()));
        asset.set_license(&Some(creation_asset.license.clone()));

        asset.set_longitude(&creation_asset.longitude);
        asset.set_latitude(&creation_asset.latitude);

        asset.set_father(&creation_asset.father);

        let res = self.repository.add(&asset, user_id).await?;
        Ok(res)
    }

    #[tracing::instrument()]
    async fn update(&self, id: &Uuid, asset: &UpdatableFildsAsset) -> ResultE<()> {
        asset.validate()?;

        let dbasset = self.repository.get_by_id(id).await?;
        let mut res: Asset = dbasset.clone();

        match &asset.license {
            None => (),
            Some(val) => res.set_license(&Some(val.to_string())),
        }

        match &asset.status {
            None => (),
            Some(sts) => {
                let aux = AssetStatus::from_str(&sts);
                match aux {
                    Err(_) => {}
                    Ok(sts_val) => res.set_state(&sts_val),
                }
            }
        }

        self.repository.update(&id, &res).await?;
        Ok(())
    }

    #[tracing::instrument()]
    async fn mint_status(
        &self,
        id: &Uuid,
        transaction: &Option<String>,
        sts: MintingStatus,
    ) -> ResultE<()> {
        let dbasset = self.repository.get_by_id(id).await?;
        let mut res: Asset = dbasset.clone();

        //res.set_minted_tx(&Some(transaction.to_owned()));
        let aux = transaction.to_owned();
        res.set_minted_tx(&aux);
        res.set_minted_status(sts);

        self.repository.update(&id, &res).await?;
        Ok(())
    }

    #[tracing::instrument()]
    async fn get_by_user_id(&self, user_id: &String) -> ResultE<Vec<Asset>> {
        let res = self.repository.get_by_user_id(user_id).await?;
        Ok(res)
    }
    #[tracing::instrument()]
    async fn get_by_user_asset_id(&self, asset_id: &Uuid, user_id: &String) -> ResultE<Asset> {
        let res = self
            .repository
            .get_by_user_asset_id(asset_id, user_id)
            .await?;
        Ok(res)
    }
    #[tracing::instrument()]
    async fn store_video_process(&self, video_res: &VideoResult) -> ResultE<()> {

        let mut original_asset = self.repository.get_by_id(&video_res.asset_id).await?;
        
        if video_res.video_op.unwrap() {
        
            let mut new_licensed_asset = Asset::new();

            new_licensed_asset.set_id(&video_res.video_licensed_asset_id.unwrap());
            new_licensed_asset.set_state(original_asset.state());
            new_licensed_asset.set_longitude(original_asset.longitude());
            new_licensed_asset.set_latitude(original_asset.latitude());
            new_licensed_asset.set_license(original_asset.license());
            new_licensed_asset.set_hash(&video_res.video_licensed_hash);
            new_licensed_asset.set_url(&video_res.video_licensed);
            new_licensed_asset.set_last_update_time(&Utc::now());
            new_licensed_asset.set_creation_time(&Utc::now());
            new_licensed_asset.set_minted_status( MintingStatus::NeverMinted);
            new_licensed_asset.set_minted_tx(&None);
            new_licensed_asset.set_video_licensing_status(VideoLicensingStatus::AlreadyLicensed);
            new_licensed_asset.set_counter(&Some(video_res.counter));
            new_licensed_asset.set_shorter(&Some(video_res.clone().shorter));
            new_licensed_asset.set_father(&Some(video_res.asset_id));

            self.repository
                .add(&new_licensed_asset, &video_res.user_id)
                .await?;

            if video_res.keep_original {
                //we need to update the original asset with new documents placed in the final location
                original_asset.set_url(&video_res.video_original);
                original_asset.set_hash(&video_res.video_original_hash);
            }
        }
        original_asset.set_video_licensing_error(&video_res.video_error);
        match video_res.video_op{
            None => {},
            Some(value)=>{
                let state = if value {VideoLicensingStatus::CompletedSuccessfully} else {VideoLicensingStatus::Error};
                original_asset.set_video_licensing_status( state );
            }
        }

        self.repository
            .update(original_asset.id(), &original_asset)
            .await?;

        Ok(())

    }

}

impl Clone for AssetService {
    #[tracing::instrument()]
    fn clone(&self) -> AssetService {
        let aux = AssetService {
            repository: self.repository.clone(),
            //owner_service: self.owner_service.clone()
        };
        return aux;
    }
}
