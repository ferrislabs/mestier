use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    ApprenticeId, ClosureReason, ClosureReasonId, CompanyTutor, CompanyTutorId, FileKey,
    HostCompanyId, MemberId, OrganizationId, PromotionId,
};

pub mod access;
pub mod commands;
pub mod events;
pub mod ports;
pub mod service;
pub mod views;

pub use access::{
    ParticipantScope, TriadAccess, TriadLink, TriadRole, TutorTriadLink, merge_links,
};
pub use views::{
    ContractTutorView, ContractView, MyTriad, MyTriadDetail, TriadDetail, TriadSummary,
};

macro_rules! triad_id {
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

triad_id!(TriadId);
triad_id!(ContractId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContractKind {
    Apprenticeship,
    Professionalization,
}

impl ContractKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Apprenticeship => "APPRENTICESHIP",
            Self::Professionalization => "PROFESSIONALIZATION",
        }
    }
}

impl FromStr for ContractKind {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "APPRENTICESHIP" => Ok(Self::Apprenticeship),
            "PROFESSIONALIZATION" => Ok(Self::Professionalization),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TutorRank {
    Main,
    Secondary,
}

impl TutorRank {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Main => "MAIN",
            Self::Secondary => "SECONDARY",
        }
    }
}

impl FromStr for TutorRank {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "MAIN" => Ok(Self::Main),
            "SECONDARY" => Ok(Self::Secondary),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContractFileKind {
    Cerfa,
    MissionSheet,
}

impl ContractFileKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cerfa => "cerfa",
            Self::MissionSheet => "mission_sheet",
        }
    }
}

impl Display for ContractFileKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContractPeriod {
    start: NaiveDate,
    end: NaiveDate,
}

impl ContractPeriod {
    pub fn new(start: NaiveDate, end: NaiveDate) -> Result<Self, CoreError> {
        if end < start {
            return Err(CoreError::Conflict(
                "contract end date cannot precede its start date".to_owned(),
            ));
        }

        Ok(Self { start, end })
    }

    pub fn start(&self) -> NaiveDate {
        self.start
    }

    pub fn end(&self) -> NaiveDate {
        self.end
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Closure {
    pub closed_at: NaiveDate,
    pub reason_id: ClosureReasonId,
    pub note: Option<String>,
}

impl Closure {
    pub fn new(
        organization_id: OrganizationId,
        closed_at: NaiveDate,
        reason: &ClosureReason,
        note: Option<String>,
    ) -> Result<Self, CoreError> {
        if reason.organization_id != organization_id {
            return Err(CoreError::NotFound);
        }
        if reason.archived_at.is_some() {
            return Err(CoreError::Conflict(
                "closure reason is archived and cannot be used for a new closure".to_owned(),
            ));
        }

        Ok(Self {
            closed_at,
            reason_id: reason.id,
            note: note
                .map(|text| text.trim().to_owned())
                .filter(|text| !text.is_empty()),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Triad {
    pub id: TriadId,
    pub organization_id: OrganizationId,
    pub promotion_id: PromotionId,
    pub apprentice_id: ApprenticeId,
    pub pedagogic_tutor_member_id: Option<MemberId>,
    pub closure: Option<Closure>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Triad {
    pub fn is_closed(&self) -> bool {
        self.closure.is_some()
    }

    pub fn ensure_open(&self) -> Result<(), CoreError> {
        if self.is_closed() {
            return Err(CoreError::Conflict("triad is closed".to_owned()));
        }

        Ok(())
    }

    pub fn close(&mut self, closure: Closure, now: DateTime<Utc>) -> Result<(), CoreError> {
        self.ensure_open()?;
        self.closure = Some(closure);
        self.updated_at = now;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Contract {
    pub id: ContractId,
    pub organization_id: OrganizationId,
    pub triad_id: TriadId,
    pub host_company_id: HostCompanyId,
    pub kind: ContractKind,
    pub number: Option<String>,
    pub period: ContractPeriod,
    pub closure: Option<Closure>,
    pub cerfa_file_key: Option<FileKey>,
    pub mission_sheet_file_key: Option<FileKey>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Contract {
    pub fn is_closed(&self) -> bool {
        self.closure.is_some()
    }

    pub fn ensure_open(&self) -> Result<(), CoreError> {
        if self.is_closed() {
            return Err(CoreError::Conflict("contract is closed".to_owned()));
        }

        Ok(())
    }

    pub fn close(&mut self, closure: Closure, now: DateTime<Utc>) -> Result<(), CoreError> {
        self.ensure_open()?;
        self.closure = Some(closure);
        self.updated_at = now;

        Ok(())
    }

    pub fn file_key(&self, kind: ContractFileKind) -> Option<&FileKey> {
        match kind {
            ContractFileKind::Cerfa => self.cerfa_file_key.as_ref(),
            ContractFileKind::MissionSheet => self.mission_sheet_file_key.as_ref(),
        }
    }

    pub fn assign_tutors(
        &self,
        main: Option<&CompanyTutor>,
        secondary: Option<&CompanyTutor>,
    ) -> Result<Vec<ContractTutor>, CoreError> {
        self.ensure_open()?;
        if let (Some(first), Some(second)) = (main, secondary)
            && first.id == second.id
        {
            return Err(CoreError::Conflict(
                "a contract cannot have the same tutor as main and secondary".to_owned(),
            ));
        }

        [(TutorRank::Main, main), (TutorRank::Secondary, secondary)]
            .into_iter()
            .filter_map(|(rank, tutor)| tutor.map(|tutor| (rank, tutor)))
            .map(|(rank, tutor)| {
                if tutor.organization_id != self.organization_id
                    || tutor.host_company_id != self.host_company_id
                {
                    return Err(CoreError::Conflict(
                        "a contract tutor must belong to the contract's host company".to_owned(),
                    ));
                }

                Ok(ContractTutor {
                    contract_id: self.id,
                    company_tutor_id: tutor.id,
                    organization_id: self.organization_id,
                    rank,
                    archived_at: None,
                })
            })
            .collect()
    }

    pub fn set_file_key(&mut self, kind: ContractFileKind, key: Option<FileKey>) {
        match kind {
            ContractFileKind::Cerfa => self.cerfa_file_key = key,
            ContractFileKind::MissionSheet => self.mission_sheet_file_key = key,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractTutor {
    pub contract_id: ContractId,
    pub company_tutor_id: CompanyTutorId,
    pub organization_id: OrganizationId,
    pub rank: TutorRank,
    pub archived_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests;
