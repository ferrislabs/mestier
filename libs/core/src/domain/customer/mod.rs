use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::OrganizationId;

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct CustomerId(pub Uuid);

impl FromStr for CustomerId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(CustomerId)
    }
}

impl Display for CustomerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CustomerStatus {
    Prospect,
    Client,
    Archived,
}

impl CustomerStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Prospect => "PROSPECT",
            Self::Client => "CLIENT",
            Self::Archived => "ARCHIVED",
        }
    }
}

impl Display for CustomerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for CustomerStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PROSPECT" => Ok(Self::Prospect),
            "CLIENT" => Ok(Self::Client),
            "ARCHIVED" => Ok(Self::Archived),
            other => Err(format!("invalid customer status `{other}`")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Customer {
    pub id: CustomerId,
    pub organization_id: OrganizationId,
    pub status: CustomerStatus,
    pub last_name: String,
    pub first_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn customer_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = CustomerId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn customer_id_rejects_invalid_uuid() {
        assert!(CustomerId::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn customer_status_parses_known_values() {
        assert_eq!(
            "PROSPECT".parse::<CustomerStatus>().unwrap(),
            CustomerStatus::Prospect
        );
        assert_eq!(
            "CLIENT".parse::<CustomerStatus>().unwrap(),
            CustomerStatus::Client
        );
    }

    #[test]
    fn customer_status_rejects_unknown_values() {
        assert!("LEAD".parse::<CustomerStatus>().is_err());
    }
}
