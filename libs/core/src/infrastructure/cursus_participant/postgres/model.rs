use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::{
    Apprentice, ApprenticeId, CompanyTutor, CompanyTutorId, CompanyTutorRecord, HostCompanyId,
    OrganizationId, Participant, ParticipantId, UserId,
};

#[derive(Debug, Clone)]
pub struct ParticipantRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub user_id: Option<Uuid>,
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

impl From<ParticipantRow> for Participant {
    fn from(row: ParticipantRow) -> Self {
        Self {
            id: ParticipantId(row.id),
            organization_id: OrganizationId(row.org_id),
            user_id: row.user_id.map(UserId),
            last_name: row.last_name,
            first_name: row.first_name,
            email: row.email,
            phone: row.phone,
            invitation_token_hash: row.invitation_token_hash,
            invitation_expires_at: row.invitation_expires_at,
            deactivated_at: row.deactivated_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApprenticeRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub participant_id: Uuid,
    pub birth_date: Option<NaiveDate>,
    pub ine_code: Option<String>,
    pub address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ApprenticeRow> for Apprentice {
    fn from(row: ApprenticeRow) -> Self {
        Self {
            id: ApprenticeId(row.id),
            organization_id: OrganizationId(row.org_id),
            participant_id: ParticipantId(row.participant_id),
            birth_date: row.birth_date,
            ine_code: row.ine_code,
            address: row.address,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompanyTutorRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub participant_id: Uuid,
    pub host_company_id: Uuid,
    pub job_title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CompanyTutorRow> for CompanyTutor {
    fn from(row: CompanyTutorRow) -> Self {
        Self {
            id: CompanyTutorId(row.id),
            organization_id: OrganizationId(row.org_id),
            participant_id: ParticipantId(row.participant_id),
            host_company_id: HostCompanyId(row.host_company_id),
            job_title: row.job_title,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompanyTutorRecordRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub participant_id: Uuid,
    pub host_company_id: Uuid,
    pub job_title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub host_company_name: String,
}

impl From<CompanyTutorRecordRow> for CompanyTutorRecord {
    fn from(row: CompanyTutorRecordRow) -> Self {
        Self {
            tutor: CompanyTutor {
                id: CompanyTutorId(row.id),
                organization_id: OrganizationId(row.org_id),
                participant_id: ParticipantId(row.participant_id),
                host_company_id: HostCompanyId(row.host_company_id),
                job_title: row.job_title,
                created_at: row.created_at,
                updated_at: row.updated_at,
            },
            host_company_name: row.host_company_name,
        }
    }
}
