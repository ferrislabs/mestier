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
pub struct HostCompanyId(pub Uuid);

impl FromStr for HostCompanyId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(HostCompanyId)
    }
}

impl Display for HostCompanyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostCompanyAddress {
    pub label: Option<String>,
    pub line1: String,
    pub line2: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub is_main: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HostCompany {
    pub id: HostCompanyId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub group_name: Option<String>,
    pub siret: Option<String>,
    pub legal_form: Option<String>,
    pub naf_code: Option<String>,
    pub capital: Option<String>,
    pub employee_count: Option<i32>,
    pub group_employee_count: Option<i32>,
    pub director_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub activity_domains: Option<String>,
    pub missions: Option<String>,
    pub addresses: Vec<HostCompanyAddress>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_company_id_parses_uuids() {
        let uuid = Uuid::new_v4();

        assert_eq!(HostCompanyId::from_str(&uuid.to_string()).unwrap().0, uuid);
    }

    #[test]
    fn host_company_id_rejects_invalid_uuids() {
        assert!(HostCompanyId::from_str("not-a-uuid").is_err());
    }
}
