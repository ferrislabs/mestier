use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::CustomerId;

pub mod commands;
pub mod ports;
pub mod service;

pub use service::CustomerContextService;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct CustomerContextId(pub Uuid);

impl FromStr for CustomerContextId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(CustomerContextId)
    }
}

impl Display for CustomerContextId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub const DEFAULT_COUNTRY: &str = "FR";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AddressKind {
    Billing,
    #[default]
    Site,
    Other,
}

impl AddressKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Billing => "BILLING",
            Self::Site => "SITE",
            Self::Other => "OTHER",
        }
    }
}

impl FromStr for AddressKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BILLING" => Ok(Self::Billing),
            "SITE" => Ok(Self::Site),
            "OTHER" => Ok(Self::Other),
            other => Err(format!("invalid address kind `{other}`")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomerContext {
    pub id: CustomerContextId,
    pub customer_id: CustomerId,
    pub label: String,
    pub kind: AddressKind,
    pub address_line: Option<String>,
    pub address_line2: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub country: String,
    pub photo_key: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn customer_context_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = CustomerContextId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn address_kind_parses_its_wire_values_and_defaults_to_site() {
        for (value, kind) in [
            ("BILLING", AddressKind::Billing),
            ("SITE", AddressKind::Site),
            ("OTHER", AddressKind::Other),
        ] {
            assert_eq!(value.parse::<AddressKind>().unwrap(), kind);
            assert_eq!(kind.as_str(), value);
        }
        assert!("HOME".parse::<AddressKind>().is_err());
        assert_eq!(AddressKind::default(), AddressKind::Site);
    }

    #[test]
    fn customer_context_id_rejects_invalid_uuid() {
        assert!(CustomerContextId::from_str("not-a-uuid").is_err());
    }
}
