use std::str::FromStr;

use crate::models::asset::{Asset, AssetStatus };
use crate::repositories::assets::{AssetRepository, AssetRepo};
use async_trait::async_trait;
use uuid::Uuid;

use validator::Validate;
type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[async_trait]
pub trait AssetManipulation {
    async fn get_all(&self, page_number: u32, page_size: u32) -> ResultE<Vec<Asset>>;
    async fn get_by_id(&self, id: &Uuid) -> ResultE<Asset>;
    async fn add(&self, asset: &mut Asset) -> ResultE<Uuid>;
    async fn update(&self, id: &Uuid, asset: &UpdatableFildsAsset) -> ResultE<()>;
}

#[derive(Debug)]
pub struct AssetService {
    repository: AssetRepo,
}

impl AssetService {
    pub fn new(repo: AssetRepo) -> AssetService {
        AssetService { repository: repo }
    }
}

#[derive(Debug,Validate)]
pub struct UpdatableFildsAsset {

    #[validate(length(max=100))]
    pub license: Option<String>,
    #[validate(length(max=10))]
    pub status: Option<String>,
}


#[async_trait]
impl AssetManipulation for AssetService {
    #[tracing::instrument()]
    async fn get_all(&self, page_number: u32, page_size: u32) -> ResultE<Vec<Asset>> {
        let res = self.repository.get_all(page_number, page_size).await?;
        Ok(res)
    }

    #[tracing::instrument()]
    async fn get_by_id(&self, id: &Uuid) -> ResultE<Asset> {
        let res = self.repository.get_by_id(id).await?;
        Ok(res)
    }

    #[tracing::instrument()]
    async fn add(&self, asset: &mut Asset) -> ResultE<Uuid> {

        let id = Uuid::new_v4();
        asset.set_id(&id);
        asset.validate()?;
        let res = self.repository.add(asset).await?;
        Ok(res)
    }

    #[tracing::instrument()]
    async fn update(&self, id: &Uuid, asset: &UpdatableFildsAsset) -> ResultE<()> {
        
        asset.validate()?;
        
        let dbasset = self.repository.get_by_id(id).await?;
        let mut res: Asset = dbasset.clone();


        match &asset.license {
            None => (),
            Some(val) => res.set_license(& Some(val.to_string())),
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

    
}

impl Clone for AssetService {
    #[tracing::instrument()]
    fn clone(&self) -> AssetService {
        let aux = AssetService {
            repository: self.repository.clone(),
        };
        return aux;
    }
}
