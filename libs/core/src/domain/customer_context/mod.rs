use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::CustomerId;

pub mod commands;
pub mod ports;
pub mod service;

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

#[derive(Debug, Clone, PartialEq)]
pub struct CustomerContext {
    pub id: CustomerContextId,
    pub customer_id: CustomerId,
    pub label: String,
    pub address_line: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
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
    fn customer_context_id_rejects_invalid_uuid() {
        assert!(CustomerContextId::from_str("not-a-uuid").is_err());
    }
}
