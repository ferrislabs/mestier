use chrono::{DateTime, NaiveDate, Utc};
use mestier_core::{
    Apprentice, ApprenticeId, ClosureReason, ClosureReasonId, CompanyTutorId, CompanyTutorRecord,
    ContractId, ContractTutorView, ContractView, CursusContractKind, CursusSpace, Establishment,
    EstablishmentId, HostCompany, HostCompanyId, MemberId, MyTriad, MyTriadDetail, OrganizationId,
    Participant, ParticipantDetail, ParticipantId, ParticipantInvite, ParticipantListItem,
    ParticipantStatus, Promotion, PromotionId, Training, TrainingId, Triad, TriadAccess,
    TriadDetail, TriadId, TriadRole, TriadSummary, TutorRank, UserId,
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
pub struct ParticipantListItemResponse {
    #[serde(flatten)]
    pub participant: ParticipantResponse,
    pub apprentice_id: Option<ApprenticeId>,
    pub company_tutors: Vec<CompanyTutorResponse>,
}

impl From<ParticipantListItem> for ParticipantListItemResponse {
    fn from(value: ParticipantListItem) -> Self {
        Self {
            participant: value.participant.into(),
            apprentice_id: value.apprentice_id,
            company_tutors: value.company_tutors.into_iter().map(Into::into).collect(),
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
    pub is_member: bool,
}

impl From<CursusSpace> for CursusSpaceResponse {
    fn from(value: CursusSpace) -> Self {
        Self {
            organization_id: value.organization_id,
            slug: value.slug,
            name: value.name,
            is_member: value.is_member,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct TriadResponse {
    pub id: TriadId,
    pub organization_id: OrganizationId,
    pub promotion_id: PromotionId,
    pub apprentice_id: ApprenticeId,
    pub pedagogic_tutor_member_id: Option<MemberId>,
    pub closed_at: Option<NaiveDate>,
    pub closure_reason_id: Option<ClosureReasonId>,
    pub closure_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Triad> for TriadResponse {
    fn from(value: Triad) -> Self {
        let (closed_at, closure_reason_id, closure_note) = match value.closure {
            Some(closure) => (
                Some(closure.closed_at),
                Some(closure.reason_id),
                closure.note,
            ),
            None => (None, None, None),
        };

        Self {
            id: value.id,
            organization_id: value.organization_id,
            promotion_id: value.promotion_id,
            apprentice_id: value.apprentice_id,
            pedagogic_tutor_member_id: value.pedagogic_tutor_member_id,
            closed_at,
            closure_reason_id,
            closure_note,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct TriadSummaryResponse {
    #[serde(flatten)]
    pub triad: TriadResponse,
    pub promotion_label: String,
    pub training_name: String,
    pub apprentice_name: String,
    pub pedagogic_tutor_name: Option<String>,
    pub current_host_company_name: Option<String>,
}

impl From<TriadSummary> for TriadSummaryResponse {
    fn from(value: TriadSummary) -> Self {
        Self {
            triad: value.triad.into(),
            promotion_label: value.promotion_label,
            training_name: value.training_name,
            apprentice_name: value.apprentice_name,
            pedagogic_tutor_name: value.pedagogic_tutor_name,
            current_host_company_name: value.current_host_company_name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ContractTutorResponse {
    pub company_tutor_id: CompanyTutorId,
    pub participant_id: ParticipantId,
    pub name: String,
    pub rank: TutorRank,
    pub archived_at: Option<DateTime<Utc>>,
}

impl From<ContractTutorView> for ContractTutorResponse {
    fn from(value: ContractTutorView) -> Self {
        Self {
            company_tutor_id: value.company_tutor_id,
            participant_id: value.participant_id,
            name: value.name,
            rank: value.rank,
            archived_at: value.archived_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ContractResponse {
    pub id: ContractId,
    pub organization_id: OrganizationId,
    pub triad_id: TriadId,
    pub host_company_id: HostCompanyId,
    pub host_company_name: String,
    pub kind: CursusContractKind,
    pub number: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub closed_at: Option<NaiveDate>,
    pub closure_reason_id: Option<ClosureReasonId>,
    pub closure_reason_label: Option<String>,
    pub closure_note: Option<String>,
    pub has_cerfa_file: bool,
    pub has_mission_sheet_file: bool,
    pub tutors: Vec<ContractTutorResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ContractView> for ContractResponse {
    fn from(value: ContractView) -> Self {
        let contract = value.contract;
        let (closed_at, closure_reason_id, closure_note) = match contract.closure {
            Some(closure) => (
                Some(closure.closed_at),
                Some(closure.reason_id),
                closure.note,
            ),
            None => (None, None, None),
        };

        Self {
            id: contract.id,
            organization_id: contract.organization_id,
            triad_id: contract.triad_id,
            host_company_id: contract.host_company_id,
            host_company_name: value.host_company_name,
            kind: contract.kind,
            number: contract.number,
            start_date: contract.period.start(),
            end_date: contract.period.end(),
            closed_at,
            closure_reason_id,
            closure_reason_label: value.closure_reason_label,
            closure_note,
            has_cerfa_file: contract.cerfa_file_key.is_some(),
            has_mission_sheet_file: contract.mission_sheet_file_key.is_some(),
            tutors: value.tutors.into_iter().map(Into::into).collect(),
            created_at: contract.created_at,
            updated_at: contract.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ContractFileUrlResponse {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct TriadDetailResponse {
    #[serde(flatten)]
    pub triad: TriadResponse,
    pub promotion: PromotionResponse,
    pub training_name: String,
    pub apprentice_name: String,
    pub apprentice_email: String,
    pub pedagogic_tutor_name: Option<String>,
    pub closure_reason_label: Option<String>,
    pub contracts: Vec<ContractResponse>,
}

impl From<TriadDetail> for TriadDetailResponse {
    fn from(value: TriadDetail) -> Self {
        Self {
            triad: value.triad.into(),
            promotion: value.promotion.into(),
            training_name: value.training_name,
            apprentice_name: value.apprentice_name,
            apprentice_email: value.apprentice_email,
            pedagogic_tutor_name: value.pedagogic_tutor_name,
            closure_reason_label: value.closure_reason_label,
            contracts: value.contracts.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TriadRoleResponse {
    Apprentice,
    MainTutor,
    SecondaryTutor,
}

impl From<TriadRole> for TriadRoleResponse {
    fn from(value: TriadRole) -> Self {
        match value {
            TriadRole::Apprentice => Self::Apprentice,
            TriadRole::MainTutor => Self::MainTutor,
            TriadRole::SecondaryTutor => Self::SecondaryTutor,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TriadAccessResponse {
    Active,
    ReadOnly { since: DateTime<Utc> },
}

impl From<TriadAccess> for TriadAccessResponse {
    fn from(value: TriadAccess) -> Self {
        match value {
            TriadAccess::Active => Self::Active,
            TriadAccess::ReadOnly { since } => Self::ReadOnly { since },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct MyTriadResponse {
    pub triad_id: TriadId,
    pub promotion_label: String,
    pub training_name: String,
    pub apprentice_name: String,
    pub role: TriadRoleResponse,
    pub access: TriadAccessResponse,
    pub closed_at: Option<NaiveDate>,
}

impl From<MyTriad> for MyTriadResponse {
    fn from(value: MyTriad) -> Self {
        Self {
            triad_id: value.summary.triad.id,
            promotion_label: value.summary.promotion_label,
            training_name: value.summary.training_name,
            apprentice_name: value.summary.apprentice_name,
            role: value.link.role.into(),
            access: value.link.access.into(),
            closed_at: value.summary.triad.closure.map(|closure| closure.closed_at),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct MyContractTutorResponse {
    pub name: String,
    pub rank: TutorRank,
    pub archived_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct MyContractResponse {
    pub id: ContractId,
    pub host_company_name: String,
    pub kind: CursusContractKind,
    pub number: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub closed_at: Option<NaiveDate>,
    pub closure_reason_label: Option<String>,
    pub closure_note: Option<String>,
    pub tutors: Vec<MyContractTutorResponse>,
}

impl From<ContractView> for MyContractResponse {
    fn from(value: ContractView) -> Self {
        let contract = value.contract;
        let (closed_at, closure_note) = match contract.closure {
            Some(closure) => (Some(closure.closed_at), closure.note),
            None => (None, None),
        };

        Self {
            id: contract.id,
            host_company_name: value.host_company_name,
            kind: contract.kind,
            number: contract.number,
            start_date: contract.period.start(),
            end_date: contract.period.end(),
            closed_at,
            closure_reason_label: value.closure_reason_label,
            closure_note,
            tutors: value
                .tutors
                .into_iter()
                .map(|tutor| MyContractTutorResponse {
                    name: tutor.name,
                    rank: tutor.rank,
                    archived_at: tutor.archived_at,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct MyTriadDetailResponse {
    pub triad_id: TriadId,
    pub promotion_id: PromotionId,
    pub promotion_label: String,
    pub promotion_start_year: i32,
    pub promotion_end_year: i32,
    pub training_name: String,
    pub apprentice_name: String,
    pub apprentice_email: String,
    pub pedagogic_tutor_name: Option<String>,
    pub closed_at: Option<NaiveDate>,
    pub closure_reason_label: Option<String>,
    pub closure_note: Option<String>,
    pub contracts: Vec<MyContractResponse>,
    pub role: TriadRoleResponse,
    pub access: TriadAccessResponse,
}

impl From<MyTriadDetail> for MyTriadDetailResponse {
    fn from(value: MyTriadDetail) -> Self {
        let detail = value.detail;
        let (closed_at, closure_note) = match detail.triad.closure {
            Some(closure) => (Some(closure.closed_at), closure.note),
            None => (None, None),
        };

        Self {
            triad_id: detail.triad.id,
            promotion_id: detail.promotion.id,
            promotion_label: detail.promotion.label,
            promotion_start_year: detail.promotion.start_year,
            promotion_end_year: detail.promotion.end_year,
            training_name: detail.training_name,
            apprentice_name: detail.apprentice_name,
            apprentice_email: detail.apprentice_email,
            pedagogic_tutor_name: detail.pedagogic_tutor_name,
            closed_at,
            closure_reason_label: detail.closure_reason_label,
            closure_note,
            contracts: detail.contracts.into_iter().map(Into::into).collect(),
            role: value.link.role.into(),
            access: value.link.access.into(),
        }
    }
}
