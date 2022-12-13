use serde::{Deserialize, Serialize};
use std::time::SystemTime;

pub trait Userer {
    fn check_login(&self) -> bool;
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct User {
    user_id: String,
    creation_time: SystemTime,
    wallet_address: String,
    email: String,
    device: String,
}

impl User {
    pub fn new() -> User {
        User {
            user_id: String::new(),
            creation_time: SystemTime::now(),
            wallet_address: String::new(),
            email: String::new(),
            device: String::new(),
        }
    }

    pub fn get_user_id(&self) -> &String {
        &self.user_id
    }
    pub fn set_user_id(&mut self) -> &mut String {
        &mut self.user_id 
    }
    pub fn get_creation_time(&self) -> &SystemTime {
        &self.creation_time
    }
    pub fn set_creation_time(&mut self ) -> &mut SystemTime {
        &mut self.creation_time 
    }
    pub fn get_wallet_address(&self) -> &String {
        &self.wallet_address
    }
    pub fn set_wallet_address(&mut self) -> &mut String {
        &mut self.wallet_address
    }
    pub fn get_email(&self) -> &String {
        &self.email
    }
    pub fn set_email(&mut self, ) -> &mut String {
        &mut self.email
    }
    pub fn get_device(&self) -> &String {
        &self.device
    }
    pub fn set_device(&mut self) -> &mut String{
        &mut self.device
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
