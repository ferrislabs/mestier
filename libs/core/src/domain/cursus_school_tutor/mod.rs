use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{MemberId, OrganizationId};

pub mod commands;
pub mod events;
pub mod ports;
pub mod service;

pub const NOT_A_SCHOOL_TUTOR: &str = "E_NOT_A_SCHOOL_TUTOR";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct SchoolTutorId(pub Uuid);

impl FromStr for SchoolTutorId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(SchoolTutorId)
    }
}

impl Display for SchoolTutorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchoolTutor {
    pub id: SchoolTutorId,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub job_title: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SchoolTutor {
    pub fn is_active(&self) -> bool {
        self.archived_at.is_none()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchoolTutorView {
    pub tutor: SchoolTutor,
    pub member_name: String,
    pub member_email: Option<String>,
    pub open_triad_count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn school_tutor_id_parses_uuids() {
        let uuid = Uuid::new_v4();

        assert_eq!(SchoolTutorId::from_str(&uuid.to_string()).unwrap().0, uuid);
    }

    #[test]
    fn school_tutor_id_rejects_invalid_uuids() {
        assert!(SchoolTutorId::from_str("not-a-uuid").is_err());
    }
}
