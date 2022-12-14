use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub trait Userer {
    fn check_login(&self) -> bool;
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct User {
    user_id: String,
    creation_time: DateTime<Utc>,
    wallet_address: String,
    email: String,
    device: String,
}

impl User {
    pub fn new() -> User {
        User {
            user_id: String::new(),
            creation_time: Utc::now(),
            wallet_address: String::new(),
            email: String::new(),
            device: String::new(),
        }
    }

    pub fn user_id(&self) -> &String {
        &self.user_id
    }
    pub fn set_user_id(&mut self, val: &String) {
        self.user_id = val.clone() 
    }
    pub fn creation_time(&self) -> &DateTime<Utc> {
        &self.creation_time
    }
    pub fn set_creation_time(&mut self, val: &DateTime<Utc> ) {
        self.creation_time = val.clone() 
    }
    pub fn wallet_address(&self) -> &String {
        &self.wallet_address
    }
    pub fn set_wallet_address(&mut self, val: &String ) {
        self.wallet_address = val.clone()
    }
    pub fn email(&self) -> &String {
        &self.email
    }
    pub fn set_email(&mut self, val: &String ) {
        self.email = val.clone()
    }
    pub fn device(&self) -> &String {
        &self.device
    }
    pub fn set_device(&mut self, val: &String) {
        self.device = val.clone()
    }
}

impl Userer for User {
    fn check_login(&self) -> bool {
        return true;
    }
}

/*
impl Clone for User {
    fn clone(&self) -> User {
        User {
            userID: (self.userID),
            creationTime: (self.creationTime),
            walletAddress: (self.walletAddress),
            email: (self.email),
            device: (self.device),
        }
    }
}*/

/*
pub fn CreateUser() -> User {
    return User::new();

}*/
