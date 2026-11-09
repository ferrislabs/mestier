use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::OrganizationId;

pub mod commands;
pub mod ports;
pub mod service;

macro_rules! catalog_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
        pub struct $name(pub Uuid);

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::from_str(s).map($name)
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

catalog_id!(EstablishmentId);
catalog_id!(TrainingId);
catalog_id!(PromotionId);
catalog_id!(ClosureReasonId);

#[derive(Debug, Clone, PartialEq)]
pub struct Establishment {
    pub id: EstablishmentId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub uai: Option<String>,
    pub siret: Option<String>,
    pub address: Option<String>,
    pub is_main_site: bool,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Training {
    pub id: TrainingId,
    pub organization_id: OrganizationId,
    pub establishment_id: Option<EstablishmentId>,
    pub name: String,
    pub rncp_code: Option<String>,
    pub rncp_label: Option<String>,
    pub level: Option<String>,
    pub degree_type: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Promotion {
    pub id: PromotionId,
    pub organization_id: OrganizationId,
    pub training_id: TrainingId,
    pub label: String,
    pub start_year: i32,
    pub end_year: i32,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClosureReason {
    pub id: ClosureReasonId,
    pub organization_id: OrganizationId,
    pub label: String,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_ids_parse_uuids() {
        let uuid = Uuid::new_v4();

        assert_eq!(
            EstablishmentId::from_str(&uuid.to_string()).unwrap().0,
            uuid
        );
        assert_eq!(PromotionId::from_str(&uuid.to_string()).unwrap().0, uuid);
    }

    #[test]
    fn catalog_ids_reject_invalid_uuids() {
        assert!(TrainingId::from_str("not-a-uuid").is_err());
        assert!(ClosureReasonId::from_str("not-a-uuid").is_err());
    }
}
