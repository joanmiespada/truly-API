use std::time::SystemTime;

pub trait Userer {
    fn CheckLogin(&self) -> bool;
}

#[derive(Clone)]
pub struct User {
    userID: String,
    creationTime: SystemTime,
    walletAddress: String,
    email: String,
    device: String,
}

impl User {
    pub fn new() -> User {
        User {
            userID: String::new(),
            creationTime: SystemTime::now(),
            walletAddress: String::new(),
            email: String::new(),
            device: String::new(),
        }
    }
}

impl Userer for User {
    fn CheckLogin(&self) -> bool {
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
