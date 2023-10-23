use uuid::Uuid;
use chrono::prelude::*;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmedStatus {
    Enabled,
    Disabled,
}

impl FromStr for ConfirmedStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Enabled" => Ok(Self::Enabled),
            "Disabled" => Ok(Self::Disabled),
            _ => Err(format!("Invalid ConfirmedStatus value: {}", s)),
        }
    }
}

impl fmt::Display for ConfirmedStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfirmedStatus::Enabled => write!(f, "Enabled"),
            ConfirmedStatus::Disabled => write!(f, "Disabled"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: String,
    pub asset_id: Uuid,
    pub confirmed: ConfirmedStatus,
    pub creation_time: DateTime<Utc>,
    pub last_update_time: DateTime<Utc>,
}

impl Subscription {
    pub fn new(user_id: String, asset_id: Uuid, state: ConfirmedStatus ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            asset_id,
            confirmed: state, // ConfirmedStatus::Disabled,
            creation_time: Utc::now(),
            last_update_time: Utc::now(),
        }
    }

    pub fn confirm(&mut self) {
        self.confirmed = ConfirmedStatus::Enabled;
        self.last_update_time = Utc::now();
    }

    pub fn unconfirm(&mut self) {
        self.confirmed = ConfirmedStatus::Disabled;
        self.last_update_time = Utc::now();
    }
}
