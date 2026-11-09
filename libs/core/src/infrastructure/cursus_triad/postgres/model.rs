use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    ApprenticeId, Closure, ClosureReasonId, CompanyTutorId, Contract, ContractId, ContractTutor,
    ContractTutorView, ContractView, FileKey, HostCompanyId, MemberId, OrganizationId,
    ParticipantId, Promotion, PromotionId, TrainingId, Triad, TriadDetail, TriadId, TriadSummary,
    TutorRank, TutorTriadLink,
    domain::{
        cursus_triad::{ContractKind, ContractPeriod},
        member::format_person_name,
    },
};

fn closure_of(
    closed_at: Option<NaiveDate>,
    reason_id: Option<Uuid>,
    note: Option<String>,
) -> Option<Closure> {
    closed_at
        .zip(reason_id)
        .map(|(closed_at, reason_id)| Closure {
            closed_at,
            reason_id: ClosureReasonId(reason_id),
            note,
        })
}

fn rank_of(value: &str) -> Result<TutorRank, CoreError> {
    value
        .parse()
        .map_err(|()| CoreError::Internal(format!("unknown tutor rank `{value}`")))
}

#[derive(Debug, Clone)]
pub struct TriadRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub promotion_id: Uuid,
    pub apprentice_id: Uuid,
    pub pedagogic_tutor_member_id: Option<Uuid>,
    pub closed_at: Option<NaiveDate>,
    pub closure_reason_id: Option<Uuid>,
    pub closure_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TriadRow> for Triad {
    fn from(row: TriadRow) -> Self {
        Self {
            id: TriadId(row.id),
            organization_id: OrganizationId(row.org_id),
            promotion_id: PromotionId(row.promotion_id),
            apprentice_id: ApprenticeId(row.apprentice_id),
            pedagogic_tutor_member_id: row.pedagogic_tutor_member_id.map(MemberId),
            closure: closure_of(row.closed_at, row.closure_reason_id, row.closure_note),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TriadSummaryRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub promotion_id: Uuid,
    pub apprentice_id: Uuid,
    pub pedagogic_tutor_member_id: Option<Uuid>,
    pub closed_at: Option<NaiveDate>,
    pub closure_reason_id: Option<Uuid>,
    pub closure_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub promotion_label: String,
    pub training_name: String,
    pub apprentice_last_name: String,
    pub apprentice_first_name: Option<String>,
    pub tutor_last_name: Option<String>,
    pub tutor_first_name: Option<String>,
    pub current_host_company_name: Option<String>,
}

impl From<TriadSummaryRow> for TriadSummary {
    fn from(row: TriadSummaryRow) -> Self {
        Self {
            apprentice_name: format_person_name(
                &row.apprentice_last_name,
                row.apprentice_first_name.as_deref(),
            ),
            pedagogic_tutor_name: row
                .tutor_last_name
                .as_deref()
                .map(|last| format_person_name(last, row.tutor_first_name.as_deref())),
            promotion_label: row.promotion_label,
            training_name: row.training_name,
            current_host_company_name: row.current_host_company_name,
            triad: Triad {
                id: TriadId(row.id),
                organization_id: OrganizationId(row.org_id),
                promotion_id: PromotionId(row.promotion_id),
                apprentice_id: ApprenticeId(row.apprentice_id),
                pedagogic_tutor_member_id: row.pedagogic_tutor_member_id.map(MemberId),
                closure: closure_of(row.closed_at, row.closure_reason_id, row.closure_note),
                created_at: row.created_at,
                updated_at: row.updated_at,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct TriadDetailRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub promotion_id: Uuid,
    pub apprentice_id: Uuid,
    pub pedagogic_tutor_member_id: Option<Uuid>,
    pub closed_at: Option<NaiveDate>,
    pub closure_reason_id: Option<Uuid>,
    pub closure_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub training_id: Uuid,
    pub promotion_label: String,
    pub promotion_start_year: i32,
    pub promotion_end_year: i32,
    pub promotion_archived_at: Option<DateTime<Utc>>,
    pub promotion_created_at: DateTime<Utc>,
    pub promotion_updated_at: DateTime<Utc>,
    pub training_name: String,
    pub apprentice_last_name: String,
    pub apprentice_first_name: Option<String>,
    pub apprentice_email: String,
    pub tutor_last_name: Option<String>,
    pub tutor_first_name: Option<String>,
    pub closure_reason_label: Option<String>,
}

impl TriadDetailRow {
    pub fn into_detail(self, contracts: Vec<ContractView>) -> TriadDetail {
        TriadDetail {
            promotion: Promotion {
                id: PromotionId(self.promotion_id),
                organization_id: OrganizationId(self.org_id),
                training_id: TrainingId(self.training_id),
                label: self.promotion_label,
                start_year: self.promotion_start_year,
                end_year: self.promotion_end_year,
                archived_at: self.promotion_archived_at,
                created_at: self.promotion_created_at,
                updated_at: self.promotion_updated_at,
            },
            training_name: self.training_name,
            apprentice_name: format_person_name(
                &self.apprentice_last_name,
                self.apprentice_first_name.as_deref(),
            ),
            apprentice_email: self.apprentice_email,
            pedagogic_tutor_name: self
                .tutor_last_name
                .as_deref()
                .map(|last| format_person_name(last, self.tutor_first_name.as_deref())),
            closure_reason_label: self.closure_reason_label,
            contracts,
            triad: Triad {
                id: TriadId(self.id),
                organization_id: OrganizationId(self.org_id),
                promotion_id: PromotionId(self.promotion_id),
                apprentice_id: ApprenticeId(self.apprentice_id),
                pedagogic_tutor_member_id: self.pedagogic_tutor_member_id.map(MemberId),
                closure: closure_of(self.closed_at, self.closure_reason_id, self.closure_note),
                created_at: self.created_at,
                updated_at: self.updated_at,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContractRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub triad_id: Uuid,
    pub host_company_id: Uuid,
    pub kind: String,
    pub number: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub closed_at: Option<NaiveDate>,
    pub closure_reason_id: Option<Uuid>,
    pub closure_note: Option<String>,
    pub cerfa_file_key: Option<String>,
    pub mission_sheet_file_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn file_key_of(value: Option<String>) -> Result<Option<FileKey>, CoreError> {
    value
        .as_deref()
        .map(FileKey::try_from)
        .transpose()
        .map_err(|error| CoreError::Internal(format!("stored contract file key: {error}")))
}

impl TryFrom<ContractRow> for Contract {
    type Error = CoreError;

    fn try_from(row: ContractRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ContractId(row.id),
            organization_id: OrganizationId(row.org_id),
            triad_id: TriadId(row.triad_id),
            host_company_id: HostCompanyId(row.host_company_id),
            kind: row.kind.parse::<ContractKind>().map_err(|()| {
                CoreError::Internal(format!("unknown contract kind `{}`", row.kind))
            })?,
            number: row.number,
            period: ContractPeriod::new(row.start_date, row.end_date)?,
            closure: closure_of(row.closed_at, row.closure_reason_id, row.closure_note),
            cerfa_file_key: file_key_of(row.cerfa_file_key)?,
            mission_sheet_file_key: file_key_of(row.mission_sheet_file_key)?,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ContractViewRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub triad_id: Uuid,
    pub host_company_id: Uuid,
    pub kind: String,
    pub number: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub closed_at: Option<NaiveDate>,
    pub closure_reason_id: Option<Uuid>,
    pub closure_note: Option<String>,
    pub cerfa_file_key: Option<String>,
    pub mission_sheet_file_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub host_company_name: String,
    pub closure_reason_label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ContractTutorViewRow {
    pub contract_id: Uuid,
    pub company_tutor_id: Uuid,
    pub participant_id: Uuid,
    pub last_name: String,
    pub first_name: Option<String>,
    pub rank: String,
    pub archived_at: Option<DateTime<Utc>>,
}

pub fn assemble_contract_views(
    contracts: Vec<ContractViewRow>,
    tutors: Vec<ContractTutorViewRow>,
) -> Result<Vec<ContractView>, CoreError> {
    let mut tutors = tutors;
    contracts
        .into_iter()
        .map(|row| {
            let (mine, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut tutors)
                .into_iter()
                .partition(|tutor| tutor.contract_id == row.id);
            tutors = rest;

            let tutors = mine
                .into_iter()
                .map(|tutor| {
                    Ok(ContractTutorView {
                        company_tutor_id: CompanyTutorId(tutor.company_tutor_id),
                        participant_id: ParticipantId(tutor.participant_id),
                        name: format_person_name(&tutor.last_name, tutor.first_name.as_deref()),
                        rank: rank_of(&tutor.rank)?,
                        archived_at: tutor.archived_at,
                    })
                })
                .collect::<Result<Vec<_>, CoreError>>()?;

            let contract = Contract::try_from(ContractRow {
                id: row.id,
                org_id: row.org_id,
                triad_id: row.triad_id,
                host_company_id: row.host_company_id,
                kind: row.kind,
                number: row.number,
                start_date: row.start_date,
                end_date: row.end_date,
                closed_at: row.closed_at,
                closure_reason_id: row.closure_reason_id,
                closure_note: row.closure_note,
                cerfa_file_key: row.cerfa_file_key,
                mission_sheet_file_key: row.mission_sheet_file_key,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })?;

            Ok(ContractView {
                contract,
                host_company_name: row.host_company_name,
                closure_reason_label: row.closure_reason_label,
                tutors,
            })
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct ContractTutorRow {
    pub contract_id: Uuid,
    pub company_tutor_id: Uuid,
    pub org_id: Uuid,
    pub rank: String,
    pub archived_at: Option<DateTime<Utc>>,
}

impl TryFrom<ContractTutorRow> for ContractTutor {
    type Error = CoreError;

    fn try_from(row: ContractTutorRow) -> Result<Self, Self::Error> {
        Ok(Self {
            contract_id: ContractId(row.contract_id),
            company_tutor_id: CompanyTutorId(row.company_tutor_id),
            organization_id: OrganizationId(row.org_id),
            rank: rank_of(&row.rank)?,
            archived_at: row.archived_at,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TutorLinkRow {
    pub triad_id: Uuid,
    pub rank: String,
    pub archived_at: Option<DateTime<Utc>>,
}

impl TryFrom<TutorLinkRow> for TutorTriadLink {
    type Error = CoreError;

    fn try_from(row: TutorLinkRow) -> Result<Self, Self::Error> {
        Ok(Self {
            triad_id: TriadId(row.triad_id),
            rank: rank_of(&row.rank)?,
            archived_at: row.archived_at,
        })
    }
}
