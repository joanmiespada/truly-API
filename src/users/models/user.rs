use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub trait Userer {
    fn check_login(&self) -> bool;
    fn promote_to_admin(&mut self);
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UserRoles  {
    Basic,
    Admin,
}

impl UserRoles{
    pub fn is_admin(&self) -> bool
    {
        match *self {
            UserRoles::Admin => true,
            _ => false,
        }
    }

    pub fn to_vec_str(input: &Vec<UserRoles>) -> Vec<String> {

        let aux: Vec<String> = input.into_iter().map(|i| i.to_string()).collect();

        return aux;
    }
    pub fn from_vec_str(input: &Vec<String>) -> Vec<UserRoles> {

        let aux: Vec<UserRoles> = input.into_iter()
            .map(|i| UserRoles::deserialize(i) )
            .filter(|f| !f.is_none())
            .map(|t| t.unwrap())
            .collect();
        return aux;
    }
    pub fn deserialize(input: &str) -> Option<UserRoles> {
        match input {
            "Basic" => return Some(UserRoles::Basic),
            "Admin" => return Some(UserRoles::Admin),
            _ => return None
        }
    }
}

impl fmt::Display for UserRoles {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UserRoles::Basic => write!(f, "Basic"),
            UserRoles::Admin => write!(f, "Admin"),
        }
    }
}


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct User {
    user_id: String,
    creation_time: DateTime<Utc>,
    wallet_address: String,
    email: String,
    //password: String,
    device: String,
    roles: Vec<UserRoles>
}

impl User {
    pub fn new() -> User {
        User {
            user_id: String::new(),
            creation_time: Utc::now(),
            wallet_address: String::new(),
            email: String::new(),
            device: String::new(),
            roles: Vec::new(),
            //password: String::new(),
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
    pub fn roles(&self) -> &Vec<UserRoles> {
        &self.roles
    }
    pub fn set_roles(&mut self, val: &Vec<UserRoles>) {
        self.roles = val.clone()
    }
    /*pub fn set_roles2(&mut self, val: &Vec<String>) {
        let rls =UserRoles::from_vec_str(val);
        self.set_roles(&rls);
    }*/
    pub fn roles_add(&mut self, val: &UserRoles) {
        self.roles.push(val.clone());
    }
    pub fn is_admin(&self) -> bool {

        let i =self.roles.iter().filter(|r| r.is_admin()).count();
        match i{
            0 => return false,
            _ => true
        }
    }
    /* 
    pub fn password (&self) -> &String {
        &self.password
    }
    pub fn set_password(&mut self, val: &String ) {
        self.password = val.clone()
    }*/

}

impl Userer for User {
    fn check_login(&self) -> bool {
        return true;
    }
    fn promote_to_admin(&mut self) {
        if !self.is_admin() {
            self.roles.push(UserRoles::Admin);
        }
    }
}


