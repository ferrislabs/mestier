use chrono::{DateTime, NaiveDate, Utc};
use mestier_core::{
    Apprentice, ApprenticeId, ClosureReason, ClosureReasonId, CompanyTutorId, CompanyTutorRecord,
    CursusSpace, Establishment, EstablishmentId, HostCompany, HostCompanyId, OrganizationId,
    Participant, ParticipantDetail, ParticipantId, ParticipantInvite, ParticipantStatus, Promotion,
    PromotionId, Training, TrainingId, UserId,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct EstablishmentResponse {
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

impl From<Establishment> for EstablishmentResponse {
    fn from(value: Establishment) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            name: value.name,
            uai: value.uai,
            siret: value.siret,
            address: value.address,
            is_main_site: value.is_main_site,
            archived_at: value.archived_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct TrainingResponse {
    pub id: TrainingId,
    pub organization_id: OrganizationId,
    pub establishment_id: EstablishmentId,
    pub name: String,
    pub rncp_code: Option<String>,
    pub rncp_label: Option<String>,
    pub level: Option<String>,
    pub degree_type: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Training> for TrainingResponse {
    fn from(value: Training) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            establishment_id: value.establishment_id,
            name: value.name,
            rncp_code: value.rncp_code,
            rncp_label: value.rncp_label,
            level: value.level,
            degree_type: value.degree_type,
            archived_at: value.archived_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct PromotionResponse {
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

impl From<Promotion> for PromotionResponse {
    fn from(value: Promotion) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            training_id: value.training_id,
            label: value.label,
            start_year: value.start_year,
            end_year: value.end_year,
            archived_at: value.archived_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ClosureReasonResponse {
    pub id: ClosureReasonId,
    pub organization_id: OrganizationId,
    pub label: String,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<ClosureReason> for ClosureReasonResponse {
    fn from(value: ClosureReason) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            label: value.label,
            archived_at: value.archived_at,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct HostCompanyResponse {
    pub id: HostCompanyId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub siret: Option<String>,
    pub address: Option<String>,
    pub legal_form: Option<String>,
    pub naf_code: Option<String>,
    pub employee_count: Option<i32>,
    pub director_name: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<HostCompany> for HostCompanyResponse {
    fn from(value: HostCompany) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            name: value.name,
            siret: value.siret,
            address: value.address,
            legal_form: value.legal_form,
            naf_code: value.naf_code,
            employee_count: value.employee_count,
            director_name: value.director_name,
            archived_at: value.archived_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ParticipantResponse {
    pub id: ParticipantId,
    pub organization_id: OrganizationId,
    pub user_id: Option<UserId>,
    pub last_name: String,
    pub first_name: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub status: ParticipantStatus,
    pub invitation_expires_at: Option<DateTime<Utc>>,
    pub deactivated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Participant> for ParticipantResponse {
    fn from(value: Participant) -> Self {
        Self {
            status: value.status(),
            id: value.id,
            organization_id: value.organization_id,
            user_id: value.user_id,
            last_name: value.last_name,
            first_name: value.first_name,
            email: value.email,
            phone: value.phone,
            invitation_expires_at: value.invitation_expires_at,
            deactivated_at: value.deactivated_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ApprenticeResponse {
    pub id: ApprenticeId,
    pub participant_id: ParticipantId,
    pub birth_date: Option<NaiveDate>,
    pub ine_code: Option<String>,
    pub address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Apprentice> for ApprenticeResponse {
    fn from(value: Apprentice) -> Self {
        Self {
            id: value.id,
            participant_id: value.participant_id,
            birth_date: value.birth_date,
            ine_code: value.ine_code,
            address: value.address,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CompanyTutorResponse {
    pub id: CompanyTutorId,
    pub participant_id: ParticipantId,
    pub host_company_id: HostCompanyId,
    pub host_company_name: String,
    pub job_title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CompanyTutorRecord> for CompanyTutorResponse {
    fn from(value: CompanyTutorRecord) -> Self {
        Self {
            id: value.tutor.id,
            participant_id: value.tutor.participant_id,
            host_company_id: value.tutor.host_company_id,
            host_company_name: value.host_company_name,
            job_title: value.tutor.job_title,
            created_at: value.tutor.created_at,
            updated_at: value.tutor.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ParticipantDetailResponse {
    #[serde(flatten)]
    pub participant: ParticipantResponse,
    pub apprentice: Option<ApprenticeResponse>,
    pub company_tutors: Vec<CompanyTutorResponse>,
}

impl From<ParticipantDetail> for ParticipantDetailResponse {
    fn from(value: ParticipantDetail) -> Self {
        Self {
            participant: value.participant.into(),
            apprentice: value.apprentice.map(Into::into),
            company_tutors: value.company_tutors.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ParticipantInviteResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

impl From<ParticipantInvite> for ParticipantInviteResponse {
    fn from(value: ParticipantInvite) -> Self {
        Self {
            token: value.token,
            expires_at: value.expires_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CursusSpaceResponse {
    pub organization_id: OrganizationId,
    pub slug: String,
    pub name: String,
}

impl From<CursusSpace> for CursusSpaceResponse {
    fn from(value: CursusSpace) -> Self {
        Self {
            organization_id: value.organization_id,
            slug: value.slug,
            name: value.name,
        }
    }
}
