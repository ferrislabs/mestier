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
pub struct ServiceRateId(pub Uuid);

impl FromStr for ServiceRateId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(ServiceRateId)
    }
}

impl Display for ServiceRateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServiceRateUnit {
    Hour,
    Ml,
    M2,
}

impl ServiceRateUnit {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hour => "HOUR",
            Self::Ml => "ML",
            Self::M2 => "M2",
        }
    }
}

impl Display for ServiceRateUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ServiceRateUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HOUR" => Ok(Self::Hour),
            "ML" => Ok(Self::Ml),
            "M2" => Ok(Self::M2),
            other => Err(format!("invalid service rate unit `{other}`")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServiceRate {
    pub id: ServiceRateId,
    pub organization_id: OrganizationId,
    pub label: String,
    pub unit: ServiceRateUnit,
    pub rate_cents: i32,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_rate_unit_parses_known_values() {
        assert_eq!(
            "HOUR".parse::<ServiceRateUnit>().unwrap(),
            ServiceRateUnit::Hour
        );
        assert_eq!(
            "ML".parse::<ServiceRateUnit>().unwrap(),
            ServiceRateUnit::Ml
        );
        assert_eq!(
            "M2".parse::<ServiceRateUnit>().unwrap(),
            ServiceRateUnit::M2
        );
    }

    #[test]
    fn service_rate_unit_rejects_unknown_values() {
        assert!("DAY".parse::<ServiceRateUnit>().is_err());
    }

    #[test]
    fn service_rate_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = ServiceRateId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }
}
