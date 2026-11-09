use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{HostCompanyId, OrganizationId, UserId};

pub mod commands;
pub mod events;
pub mod ports;
pub mod service;

pub const INVITATION_VALIDITY_DAYS: i64 = 14;

macro_rules! participant_id {
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

participant_id!(ParticipantId);
participant_id!(ApprenticeId);
participant_id!(CompanyTutorId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantStatus {
    Pending,
    Active,
    Inactive,
}

impl ParticipantStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Inactive => "inactive",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantKind {
    Apprentice,
    CompanyTutor,
}

impl ParticipantKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Apprentice => "apprentice",
            Self::CompanyTutor => "company_tutor",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Participant {
    pub id: ParticipantId,
    pub organization_id: OrganizationId,
    pub user_id: Option<UserId>,
    pub last_name: String,
    pub first_name: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub invitation_token_hash: Option<Vec<u8>>,
    pub invitation_expires_at: Option<DateTime<Utc>>,
    pub deactivated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Participant {
    pub fn status(&self) -> ParticipantStatus {
        if self.deactivated_at.is_some() {
            ParticipantStatus::Inactive
        } else if self.user_id.is_none() {
            ParticipantStatus::Pending
        } else {
            ParticipantStatus::Active
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Apprentice {
    pub id: ApprenticeId,
    pub organization_id: OrganizationId,
    pub participant_id: ParticipantId,
    pub birth_date: Option<NaiveDate>,
    pub ine_code: Option<String>,
    pub address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompanyTutor {
    pub id: CompanyTutorId,
    pub organization_id: OrganizationId,
    pub participant_id: ParticipantId,
    pub host_company_id: HostCompanyId,
    pub job_title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompanyTutorRecord {
    pub tutor: CompanyTutor,
    pub host_company_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticipantDetail {
    pub participant: Participant,
    pub apprentice: Option<Apprentice>,
    pub company_tutors: Vec<CompanyTutorRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticipantInvite {
    pub participant: Participant,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CursusSpace {
    pub organization_id: OrganizationId,
    pub slug: String,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn participant() -> Participant {
        let now = Utc::now();
        Participant {
            id: ParticipantId(Uuid::new_v4()),
            organization_id: OrganizationId(Uuid::new_v4()),
            user_id: None,
            last_name: "Martin".to_owned(),
            first_name: None,
            email: "martin@example.com".to_owned(),
            phone: None,
            invitation_token_hash: None,
            invitation_expires_at: None,
            deactivated_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn a_participant_without_a_user_is_pending() {
        assert_eq!(participant().status(), ParticipantStatus::Pending);
    }

    #[test]
    fn a_participant_with_a_user_is_active() {
        let mut linked = participant();
        linked.user_id = Some(UserId(Uuid::new_v4()));

        assert_eq!(linked.status(), ParticipantStatus::Active);
    }

    #[test]
    fn deactivation_wins_over_every_other_state() {
        let mut pending = participant();
        pending.deactivated_at = Some(Utc::now());
        let mut linked = pending.clone();
        linked.user_id = Some(UserId(Uuid::new_v4()));

        assert_eq!(pending.status(), ParticipantStatus::Inactive);
        assert_eq!(linked.status(), ParticipantStatus::Inactive);
    }

    #[test]
    fn participant_ids_parse_uuids() {
        let uuid = Uuid::new_v4();

        assert_eq!(ParticipantId::from_str(&uuid.to_string()).unwrap().0, uuid);
        assert!(CompanyTutorId::from_str("not-a-uuid").is_err());
    }
}
