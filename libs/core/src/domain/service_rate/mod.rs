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
    /// A price for a whole job, priced once rather than per quantity.
    FlatRate,
    Hour,
    Day,
    /// A count of things: fittings, fixtures, anything sold by the piece.
    Unit,
    Ml,
    M2,
    M3,
    Kg,
    Tonne,
    Litre,
}

impl ServiceRateUnit {
    /// Every variant, ordered the way a quote form should offer them: time,
    /// then counts, then geometry, then mass and volume.
    pub const ALL: [ServiceRateUnit; 10] = [
        Self::FlatRate,
        Self::Hour,
        Self::Day,
        Self::Unit,
        Self::Ml,
        Self::M2,
        Self::M3,
        Self::Kg,
        Self::Tonne,
        Self::Litre,
    ];

    /// The wire form. Exhaustive by construction, so a new variant cannot be
    /// added without naming how it is stored.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FlatRate => "FLAT_RATE",
            Self::Hour => "HOUR",
            Self::Day => "DAY",
            Self::Unit => "UNIT",
            Self::Ml => "ML",
            Self::M2 => "M2",
            Self::M3 => "M3",
            Self::Kg => "KG",
            Self::Tonne => "TONNE",
            Self::Litre => "LITRE",
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
        ServiceRateUnit::ALL
            .into_iter()
            .find(|unit| unit.as_str() == s)
            .ok_or_else(|| format!("invalid service rate unit `{s}`"))
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

    /// Covers every variant without naming them, so widening `ALL` extends the
    /// test instead of leaving a new unit unparsed.
    #[test]
    fn every_service_rate_unit_round_trips_through_its_wire_form() {
        for unit in ServiceRateUnit::ALL {
            assert_eq!(unit.as_str().parse::<ServiceRateUnit>(), Ok(unit));
        }
    }

    /// `as_str` is hand-written and `Serialize` is derived from `rename_all`.
    /// Nothing makes the two agree, so a variant whose name does not
    /// screaming-snake-case to its wire form would ship a silent mismatch
    /// between the database column and the JSON body.
    #[test]
    fn the_serialized_form_is_the_stored_form() {
        for unit in ServiceRateUnit::ALL {
            assert_eq!(
                serde_json::to_value(unit).expect("a unit serializes"),
                serde_json::Value::String(unit.as_str().to_owned()),
            );
        }
    }

    #[test]
    fn service_rate_unit_rejects_unknown_values() {
        assert!("FURLONG".parse::<ServiceRateUnit>().is_err());
    }

    #[test]
    fn service_rate_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = ServiceRateId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }
}
