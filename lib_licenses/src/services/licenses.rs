use crate::models::license::License;
use crate::repositories::licenses::{LicenseRepo, LicenseRepository};
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[async_trait]
pub trait LicenseManipulation {
    async fn get_all(&self, page_number: u32, page_size: u32) -> ResultE<Vec<License>>;
    async fn get_by_id(&self, license_id: &Uuid, asset_id: &Uuid) -> ResultE<Option<License>>;
    async fn get_by_license(&self, license_id: &Uuid) -> ResultE<Option<License>>;
    async fn get_by_asset(&self, asset_id: &Uuid) -> ResultE<Vec<License>>;
    async fn create(&self, license: &mut License) -> ResultE<()>;
    async fn update(&self, license: &License) -> ResultE<()>;
    async fn delete(&self, id: &Uuid) -> ResultE<()>;
}

#[derive(Debug)]
pub struct LicenseService {
    repository: LicenseRepo,
}

impl LicenseService {
    pub fn new(repo: LicenseRepo) -> LicenseService {
        LicenseService { repository: repo }
    }
}

#[async_trait]
impl LicenseManipulation for LicenseService {
    async fn get_all(&self, page_number: u32, page_size: u32) -> ResultE<Vec<License>> {
        let res = self.repository.get_all(page_number, page_size).await?;
        Ok(res)
    }

    async fn get_by_id(&self, license_id: &Uuid, asset_id: &Uuid) -> ResultE<Option<License>> {
        let res = self.repository.get_by_id(license_id, asset_id ).await?;
        Ok(res)
    }
    async fn get_by_license(&self, license_id: &Uuid) -> ResultE<Option<License>> {
        let res = self.repository.get_by_license_id(license_id ).await?;
        Ok(res)
    }
    async fn get_by_asset(&self, asset_id: &Uuid) -> ResultE<Vec<License>>{
        let res = self.repository.get_by_asset_id(asset_id ).await?;
        Ok(res)
    }

    async fn create(&self, license: &mut License) -> ResultE<()> {
        license.set_id(Uuid::new_v4());
        license.set_creation_time(Utc::now());
        license.set_last_update_time(Utc::now());

        self.repository.create(license).await?;
        Ok(())
    }

    async fn update(&self, license: &License) -> ResultE<()> {
        self.repository.update(license).await?;
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> ResultE<()> {
        self.repository.delete(id).await?;
        Ok(())
    }
}

impl Clone for LicenseService {
    fn clone(&self) -> LicenseService {
        let aux = LicenseService {
            repository: self.repository.clone(),
        };
        aux
    }
}
