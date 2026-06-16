use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::OrganizationId;

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, ToSchema)]
pub struct EquipmentId(pub Uuid);

impl FromStr for EquipmentId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(EquipmentId)
    }
}

impl Display for EquipmentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Equipment {
    pub id: EquipmentId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub hourly_rate_cents: i32,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equipment_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = EquipmentId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn equipment_id_rejects_invalid_uuid() {
        assert!(EquipmentId::from_str("not-a-uuid").is_err());
    }
}
