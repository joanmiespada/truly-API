use crate::models::asset::{Asset, AssetStatus, SourceType};
use crate::models::asset::{AssetEnhanced, VideoLicensingStatus};
use crate::repositories::assets::{AssetRepo, AssetRepository};
use crate::repositories::shorter::{ShorterRepo, ShorterRepository};
use async_trait::async_trait;
use chrono::Utc;
use lib_video_objs::video::VideoResult;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::{error, info};
use url::Url;
use uuid::Uuid;

use validator::Validate;

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[async_trait]
pub trait AssetManipulation {
    async fn get_all(&self, page_number: u32, page_size: u32) -> ResultE<Vec<Asset>>;
    async fn get_by_id(&self, asset_id: &Uuid) -> ResultE<Asset>;
    async fn get_by_url(&self, url: &Url) -> ResultE<Asset>;
    async fn get_by_id_enhanced(&self, asset_id: &Uuid) -> ResultE<AssetEnhanced>;
    async fn get_by_shorter(&self, shorter_id: &String) -> ResultE<Asset>;
    async fn get_by_user_id(&self, user_id: &String) -> ResultE<Vec<Asset>>;
    async fn get_by_user_asset_id(&self, asset_id: &Uuid, user_id: &String) -> ResultE<Asset>;
    async fn add(&self, creation_asset: &CreatableFildsAsset, user_id: &String) -> ResultE<Uuid>;
    async fn update(&self, asset_id: &Uuid, asset: &UpdatableFildsAsset) -> ResultE<()>;
    async fn update_full(&self, asset: &Asset) -> ResultE<()>;
    // async fn mint_status(
    //     &self,
    //     id: &Uuid,
    //     transaction: &Option<String>,
    //     sts: MintingStatus,
    // ) -> ResultE<()>;
    async fn store_video_process(&self, video_res: &VideoResult) -> ResultE<()>;
    async fn shorter_video_status(
        &self,
        id: &Uuid,
        message: &Option<String>,
        sts: VideoLicensingStatus,
    ) -> ResultE<()>;
}

#[derive(Debug)]
pub struct AssetService {
    repository: AssetRepo,
    short_repository: ShorterRepo, //owner_service: OwnerService,
}

impl AssetService {
    pub fn new(ass_repo: AssetRepo, short_repo: ShorterRepo) -> AssetService {
        //,owner_service: OwnerService
        AssetService {
            repository: ass_repo,
            short_repository: short_repo,
        } // owner_service: owner_service.clone() }
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
    pub license: Option<String>,
    pub url: String,
    #[validate(length(max = 2000))]
    pub hash: Option<String>,
    #[validate(length(max = 2000))]
    pub hash_algorithm: Option<String>,

    pub latitude: Option<f64>,
    pub longitude: Option<f64>,

    pub father: Option<Uuid>,

    pub source: SourceType,
    #[validate(length(max = 1000))]
    pub source_details: Option<String>,
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
    async fn get_by_url(&self, url: &Url) -> ResultE<Asset> {
        let res = self.repository.get_by_url(url).await?;
        Ok(res)
    }

    #[tracing::instrument()]
    async fn get_by_id_enhanced(
        &self,
        id: &Uuid,
    ) -> std::result::Result<AssetEnhanced, Box<dyn std::error::Error + Sync + Send>> {
        let asset = self.repository.get_by_id(id).await?;
        let son_uids = self.repository.get_sons(id).await?;
        let mut sons = Vec::new();
        for son in son_uids {
            let son_ass_op = self.repository.get_by_id(&son).await;
            match son_ass_op {
                Err(_) => {
                    error!("id registered as a son has no entity! It shouldn't happen!")
                }
                Ok(son_id) => sons.push(son_id),
            }
        }
        let result = AssetEnhanced { asset, sons };
        Ok(result)
    }

    #[tracing::instrument()]
    async fn get_by_shorter(&self, shorter_id: &String) -> ResultE<Asset> {
        let res = self.short_repository.get_by_shorter(shorter_id).await?;
        let asset = self.repository.get_by_id(&res).await?;
        Ok(asset)
    }

    #[tracing::instrument()]
    async fn add(&self, creation_asset: &CreatableFildsAsset, user_id: &String) -> ResultE<Uuid> {
        creation_asset.validate()?;

        let new_intent_asset = creation_asset.url.clone();
        let urll = url::Url::parse(new_intent_asset.as_str())?;
        let res_op = self.repository.get_by_url(&urll).await;
        if let Ok(_) = res_op {
            return Err(format!("asset with url {} already exists", urll).into());
        }

        info!("asset fields validated");
        let mut asset = Asset::new();
        asset.set_state(&AssetStatus::Enabled);
        asset.set_id(&Uuid::new_v4());
        asset.set_url(&Some(urll));
        if let Some(hash) = creation_asset.clone().hash {
            asset.set_hash(&Some( hash.clone()));
        }else{
            asset.set_hash(&None);
        }
        if let Some(hash_algorithm) = creation_asset.clone().hash_algorithm {
            asset.set_hash_algorithm(&Some(hash_algorithm.clone()));
        }else{
            asset.set_hash_algorithm(&None);
        }

        asset.set_longitude(&creation_asset.longitude);
        asset.set_latitude(&creation_asset.latitude);

        asset.set_father(&creation_asset.father);

        info!("attaching new asset to repository");
        let res = self.repository.add(&asset, user_id).await?;
        Ok(res)
    }

    #[tracing::instrument()]
    async fn update(&self, id: &Uuid, asset: &UpdatableFildsAsset) -> ResultE<()> {
        asset.validate()?;

        let dbasset = self.repository.get_by_id(id).await?;
        let mut res: Asset = dbasset.clone();

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

        self.repository.update(&res).await?;
        Ok(())
    }

    #[tracing::instrument()]
    async fn update_full(&self, asset: &Asset) -> ResultE<()> {
        self.repository.update(asset).await?;
        Ok(())
    }

   /*  #[tracing::instrument()]
    async fn mint_status(
        &self,
        id: &Uuid,
        transaction: &Option<String>,
        sts: MintingStatus,
    ) -> ResultE<()> {
        let dbasset = self.repository.get_by_id(id).await?;
        let mut res: Asset = dbasset.clone();

        let aux = transaction.to_owned();
        res.set_minted_tx(&aux);
        res.set_minted_status(sts);

        self.repository.update(&res).await?;
        Ok(())
    } */

    #[tracing::instrument()]
    async fn shorter_video_status(
        &self,
        id: &Uuid,
        message: &Option<String>,
        sts: VideoLicensingStatus,
    ) -> ResultE<()> {
        let dbasset = self.repository.get_by_id(id).await?;
        let mut res: Asset = dbasset.clone();

        res.set_video_licensing_error(message);
        res.set_video_licensing_status(sts);

        self.repository.update(&res).await?;
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

        if let Some(op) = video_res.video_op {
            if op {
                let mut new_licensed_asset = Asset::new();

                new_licensed_asset.set_id(&video_res.video_licensed_asset_id.unwrap());
                new_licensed_asset.set_state(original_asset.state());
                new_licensed_asset.set_longitude(original_asset.longitude());
                new_licensed_asset.set_latitude(original_asset.latitude());
                new_licensed_asset.set_hash(&video_res.video_licensed_hash);
                new_licensed_asset.set_hash_algorithm(&video_res.video_licensed_hash_algorithm);
                new_licensed_asset.set_url(&video_res.video_licensed);
                new_licensed_asset.set_last_update_time(&Utc::now());
                new_licensed_asset.set_creation_time(&Utc::now());
                //new_licensed_asset.set_minted_status(MintingStatus::NeverMinted);
                //new_licensed_asset.set_minted_tx(&None);
                new_licensed_asset
                    .set_video_licensing_status(VideoLicensingStatus::AlreadyLicensed);
                new_licensed_asset.set_counter(&Some(video_res.counter));
                new_licensed_asset.set_shorter(&Some(video_res.clone().shorter));
                new_licensed_asset.set_father(&Some(video_res.asset_id));

                self.repository
                    .add(&new_licensed_asset, &video_res.user_id)
                    .await?;

                self.short_repository
                    .add(
                        &video_res.video_licensed_asset_id.unwrap(),
                        &video_res.clone().shorter,
                    )
                    .await?;

                if video_res.keep_original {
                    //we need to update the original asset with new documents placed in the final location
                    original_asset.set_url(&video_res.video_original);
                    original_asset.set_hash(&video_res.video_original_hash);
                }
            }
        }
        original_asset.set_video_licensing_error(&video_res.video_error);
        original_asset.set_video_process_status(&video_res.video_process_status);

        match video_res.video_op {
            None => {}
            Some(value) => {
                let state = if value {
                    VideoLicensingStatus::CompletedSuccessfully
                } else {
                    VideoLicensingStatus::Error
                };
                original_asset.set_video_licensing_status(state);
            }
        }

        self.repository.update(&original_asset).await?;

        Ok(())
    }
}

impl Clone for AssetService {
    #[tracing::instrument()]
    fn clone(&self) -> AssetService {
        let aux = AssetService {
            repository: self.repository.clone(),
            short_repository: self.short_repository.clone(),
        };
        return aux;
    }
}
