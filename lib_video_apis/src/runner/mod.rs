use derive_builder::Builder;
use lib_config::{config::Config, environment::EnvironmentVariables, result::ResultE};
use lib_licenses::{
    ops::create_asset,
    services::{
        assets::{AssetService, CreatableFildsAssetBuilder},
        video::VideoService,
    },
};
use url::Url;
use uuid::Uuid;

use crate::{
    twitch::{TwitchAPIBuilder, ID as TWITCH_ID},
    youtube::{YoutubeAPIBuilder, ID as YOUTUBE_ID},
    ExternalData,
};

#[derive(Clone, Debug, Builder)]
pub struct Runner {
    environment_vars: EnvironmentVariables,
    platform_ids: Vec<String>,
    asset_service: AssetService,
    video_service: VideoService,
}

impl Runner {
    pub fn new(conf: &Config, platform_ids: Vec<String>, asset_service: &AssetService, video_service: &VideoService  ) -> Runner {
        Runner {
            environment_vars: conf.env_vars().clone(),
            platform_ids,
            asset_service: asset_service.clone(),
            video_service: video_service.clone(),
        }
    }

    // pub fn set_asset_service(&mut self, ass: &AssetService) {
    //     self.asset_service = Some(ass.clone());
    // }
    // pub fn set_video_service(&mut self, vss: &VideoService) {
    //     self.video_service = Some(vss.clone());
    // }

    async fn create_assets(&self, items: Vec<Url>) -> ResultE<Vec<Uuid>> {
        let mut result = Vec::new();
    
        // Check if services are available
        //let asset_service = self.asset_service.as_ref().ok_or("Asset service not available")?;
        //let video_service = self.video_service.as_ref().ok_or("Video service not available")?;
    
        for item in items {
            let asset_fields = CreatableFildsAssetBuilder::default()
                .url(item.to_string())
                .build()
                .map_err(|_| format!("Url not well formatted: {}", item))?;
    
            match create_asset(&self.asset_service,&self.video_service, None, &asset_fields).await {
                Err(e) => eprintln!("Error storing urls: {}", e),
                Ok(asset_id) => result.push(asset_id),
            }
        }
    
        Ok(result)
    }

    // async fn create_assets(&self, items: Vec<Url>) -> Vec<Uuid> {
    //     let mut result = Vec::new();
    //     for item in items {
    //         let asset_fields_op = CreatableFildsAssetBuilder::default()
    //             .url(item.to_string())
    //             .build();
    //         match asset_fields_op {
    //             Ok(asset_fields) => {
    //                 let op = create_asset(
    //                     &self.asset_service.unwrap(),
    //                     &self.video_service.unwrap(),
    //                     None,
    //                     &asset_fields,
    //                 )
    //                 .await;
    //                 match op {
    //                     Err(e) => println!("error storing urls: {}", e),
    //                     Ok(asset_id) => result.push(asset_id),
    //                 }
    //             }
    //             Err(e) => println!("url not well formated {}", item),
    //         }
    //     }
    //     Ok(result)
    // }

    pub async fn process_searches(&self, word_keys: Vec<String>) -> ResultE<()> {
        let mut platforms: Vec<Box<dyn ExternalData>> = Vec::new();

        for plat in &self.platform_ids {
            match plat.as_str() {
                YOUTUBE_ID => platforms.push(Box::new(
                    YoutubeAPIBuilder::default()
                        .environment_vars(self.environment_vars.clone())
                        .build()?,
                )),
                TWITCH_ID => platforms.push(Box::new(
                    TwitchAPIBuilder::default()
                        .environment_vars(self.environment_vars.clone())
                        .build()?,
                )),
                _ => {}
            }
        }
        for word in &word_keys {
            for platform in &mut platforms {
                let res = self
                    .process_search_results(platform, word.clone(), None)
                    .await;
                if let Err(e) = res {
                    println!("Something get wrong! {}", e);
                }
            }
        }
        Ok(())
    }

    async fn process_search_results(
        &self,
        platform: &mut Box<dyn ExternalData>,
        word_key: String,
        page_token: Option<String>,
    ) -> ResultE<()> {
        let mut current_token = page_token;
        loop {
            match platform
                .search(vec![word_key.clone()], current_token, true)
                .await
            {
                Ok((urls, token)) => {
                    self.create_assets(urls).await;
                    if token.is_none() {
                        break;
                    }
                    current_token = token;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}
