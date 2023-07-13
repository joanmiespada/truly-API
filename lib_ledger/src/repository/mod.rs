use async_trait::async_trait;

use chrono::prelude::{DateTime, Utc};
use lib_config::config::Config;
use lib_licenses::models::asset::Asset;
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::Ledge;
use lib_config::result::ResultE;

//use aws_sdk_qldb::{config::Region, meta::PKG_VERSION, Client, Error};
use aws_sdk_qldbsession::{Client, types::{builders::StartSessionRequestBuilder, StartSessionRequest, ExecuteStatementRequest} };

use self::schema_ledger::LEDGER_TABLE_NAME;

pub mod schema_ledger;

#[async_trait]
pub trait LedgerRepository {
    async fn add(&self, asset: &Asset) -> ResultE<Ledge>;
    async fn get_by_id(&self, hash: &String) -> ResultE<Ledge>;
    async fn get_by_asset_id(&self, asset_id: &Uuid) -> ResultE<Vec<Ledge>>;
}

#[derive(Clone, Debug)]
pub struct LedgerRepo {
    client: Client,
}

impl LedgerRepo {
    pub fn new(conf: &Config) -> LedgerRepo {
        LedgerRepo {
            client: Client::new(conf.aws_config()),
        }
    }
}

#[async_trait]
impl LedgerRepository for LedgerRepo {
    async fn add(&self, asset: &Asset) -> ResultE<Ledge>{

        /* 
        let op = self.client.send_command().start_session(
            StartSessionRequest::builder().ledger_name(LEDGER_TABLE_NAME).build()
        ).execute_statement(
            ExecuteStatementRequest::builder()
            .statement("input")
            .build()
        ).send()
        .await;*/

        //self.client
        //    .send_command()
        Ok( Ledge::default() )
            
    }
    async fn get_by_id(&self, hash: &String) -> ResultE<Ledge>{
        Ok(Ledge::default())

    }
    async fn get_by_asset_id(&self, asset_id: &Uuid) -> ResultE<Vec<Ledge>>{
        Ok( vec![ Ledge::default() ])
    }
}



//pub fn mapping_from_doc_to_ledge(doc: &HashMap<String, AttributeValue>) -> Ledge {  
//}

