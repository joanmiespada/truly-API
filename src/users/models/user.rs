use std::time::SystemTime;
use serde::{Serialize, Deserialize};

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
