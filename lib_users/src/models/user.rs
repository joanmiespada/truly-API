use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;
use validator::Validate;


pub trait Userer {
    fn check_login(&self) -> bool;
    fn promote_to_admin(&mut self);
    fn downgrade_from_admin(&mut self);
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum UserRoles {
    Basic,
    Admin,
}

impl UserRoles {
    pub fn is_admin(&self) -> bool {
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
        let aux: Vec<UserRoles> = input
            .into_iter()
            .map(|i| UserRoles::deserialize(i))
            .filter(|f| !f.is_none())
            .map(|t| t.unwrap())
            .collect();
        return aux;
    }
    pub fn deserialize(input: &str) -> Option<UserRoles> {
        match input {
            "Basic" => return Some(UserRoles::Basic),
            "Admin" => return Some(UserRoles::Admin),
            _ => return None,
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
pub enum UserStatus {
    Enabled,
    Disabled,
}

impl UserStatus{
    pub fn is_disabled(&self) -> bool {
        match *self {
            UserStatus::Disabled => true,
            _ => false,
        }
    }
}

#[derive(Clone, Serialize,Validate, Deserialize, Debug)]
pub struct User {
    #[validate(length( max=100))]
    user_id: String,
    creation_time: DateTime<Utc>,
    last_update_time: DateTime<Utc>,
    #[validate(email)]
    email: Option<String>,
    //password: String, // don't use it here!
    #[validate(length( max=100))]
    device: Option<String>,
    roles: Vec<UserRoles>,
    status: UserStatus,
}

impl User {
    pub fn new() -> User {
        User {
            user_id: Uuid::nil().to_string(),
            creation_time: Utc::now(),
            last_update_time: Utc::now(),
            email: None,
            device: None,
            roles: Vec::new(),
            //password: String::new(),
            status: UserStatus::Enabled,
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
    pub fn set_creation_time(&mut self, val: &DateTime<Utc>) {
        self.creation_time = val.clone()
    }
    pub fn last_update_time(&self) -> &DateTime<Utc> {
        &self.last_update_time
    }
    pub fn set_last_update_time(&mut self, val: &DateTime<Utc>) {
        self.last_update_time = val.clone()
    }
    pub fn email(&self) -> &Option<String> {
        &self.email
    }
    pub fn set_email(&mut self, val: &String) {
        self.email = Some(val.clone())
    }
    pub fn device(&self) -> &Option<String> {
        &self.device
    }
    pub fn set_device(&mut self, val: &String) {
        self.device = Some(val.clone())
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
        let i = self.roles.iter().filter(|r| r.is_admin()).count();
        match i {
            0 => return false,
            _ => true,
        }
    }
    /*
    pub fn password (&self) -> &String {
        &self.password
    }
    pub fn set_password(&mut self, val: &String ) {
        self.password = val.clone()
    }*/
    pub fn status(&self) -> &UserStatus {
        &self.status
    }
    pub fn set_status(&mut self, val: &UserStatus) {
        self.status = val.clone()
    }
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
    fn downgrade_from_admin(&mut self) {
        if self.is_admin() {
            let aux = self.roles.clone();
            let roles_without_admin: Vec<UserRoles> =
                aux.into_iter().filter(|x| !x.is_admin()).collect(); // .push(UserRoles::Admin);
            self.roles = roles_without_admin.clone();
        }
    }
}

impl UserStatus {
    pub fn parse(input: &str) -> Option<UserStatus> {
        match input {
            "Enabled" => Some(UserStatus::Enabled),
            "Disabled" => Some(UserStatus::Disabled),
            _ => None, 
        }
    }
}

impl fmt::Display for UserStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UserStatus::Enabled => write!(f, "Enabled"),
            UserStatus::Disabled => write!(f, "Disabled"),
        }
    }
}
