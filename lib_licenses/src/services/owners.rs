

use crate::errors::owner::OwnerAlreadyExistsError;
use crate::models::owner::Owner;
use crate::repositories::owners::{OwnerRepository, OwnerRepo};
use async_trait::async_trait;
use uuid::Uuid;

use validator::Validate;
type ResultE<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[async_trait]
pub trait OwnerManipulation {
    async fn get_all(&self, page_number: u32, page_size: u32) -> ResultE<Vec<Owner>>;
    async fn get_by_user(&self, id: &String) -> ResultE<Vec<Owner>>;
    async fn get_by_asset(&self, id: &Uuid) -> ResultE<Owner>;
    async fn add(&self, owner: &mut Owner) -> ResultE<()>;
    async fn update(&self, current: &Owner, new_owner: &UpdatableFildsOwner) -> ResultE<()>;
}

#[derive(Debug)]
pub struct OwnerService {
    repository: OwnerRepo,
}

impl OwnerService {
    pub fn new(repo: OwnerRepo) -> OwnerService {
        OwnerService { repository: repo }
    }
}

#[derive(Debug,Validate)]
pub struct UpdatableFildsOwner {

    #[validate(length(max=100))]
    pub new_owner: Option<String>,
}


#[async_trait]
impl OwnerManipulation for OwnerService {
    #[tracing::instrument()]
    async fn get_all(&self, page_number: u32, page_size: u32) -> ResultE<Vec<Owner>> {
        let res = self.repository.get_all(page_number, page_size).await?;
        Ok(res)
    }

    #[tracing::instrument()]
    async fn get_by_asset(&self, asset_id: &Uuid) -> ResultE<Owner> {
        let res = self.repository.get_by_asset(asset_id).await?;
        Ok(res)
    }
    
    #[tracing::instrument()]
    async fn get_by_user(&self, user_id: &String) -> ResultE<Vec<Owner>> {
        let res = self.repository.get_by_user(user_id).await?;
        Ok(res)
    }

    #[tracing::instrument()]
    async fn add(&self, owner: &mut Owner) -> ResultE<()> {

        owner.validate()?;
        self.repository.add(owner).await?;
        Ok(())
    }

    async fn update(&self, current_owner: &Owner, new_owner: &UpdatableFildsOwner) -> ResultE<()> {
      
        new_owner.validate()?;
        let aux = new_owner.new_owner.clone().unwrap();
        
        self.repository.update(current_owner, &aux ).await?;
        
        Ok(())
    }
}

impl Clone for OwnerService {
    #[tracing::instrument()]
    fn clone(&self) -> OwnerService {
        let aux = OwnerService {
            repository: self.repository.clone(),
        };
        return aux;
    }
}
