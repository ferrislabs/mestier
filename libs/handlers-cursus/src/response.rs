use chrono::{DateTime, Utc};
use mestier_core::{
    ClosureReason, ClosureReasonId, Establishment, EstablishmentId, HostCompany, HostCompanyId,
    OrganizationId, Promotion, PromotionId, Training, TrainingId,
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
