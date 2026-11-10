use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    ClosureReason, ClosureReasonId, Establishment, EstablishmentId, OrganizationId, Promotion,
    PromotionId, Training, TrainingId,
};

#[derive(Debug, Clone)]
pub struct EstablishmentRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub uai: Option<String>,
    pub siret: Option<String>,
    pub address: Option<String>,
    pub is_main_site: bool,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<EstablishmentRow> for Establishment {
    fn from(row: EstablishmentRow) -> Self {
        Self {
            id: EstablishmentId(row.id),
            organization_id: OrganizationId(row.org_id),
            name: row.name,
            uai: row.uai,
            siret: row.siret,
            address: row.address,
            is_main_site: row.is_main_site,
            archived_at: row.archived_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrainingRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub establishment_id: Uuid,
    pub name: String,
    pub rncp_code: Option<String>,
    pub rncp_label: Option<String>,
    pub level: Option<String>,
    pub degree_type: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TrainingRow> for Training {
    fn from(row: TrainingRow) -> Self {
        Self {
            id: TrainingId(row.id),
            organization_id: OrganizationId(row.org_id),
            establishment_id: EstablishmentId(row.establishment_id),
            name: row.name,
            rncp_code: row.rncp_code,
            rncp_label: row.rncp_label,
            level: row.level,
            degree_type: row.degree_type,
            archived_at: row.archived_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PromotionRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub training_id: Uuid,
    pub label: String,
    pub start_year: i32,
    pub end_year: i32,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PromotionRow> for Promotion {
    fn from(row: PromotionRow) -> Self {
        Self {
            id: PromotionId(row.id),
            organization_id: OrganizationId(row.org_id),
            training_id: TrainingId(row.training_id),
            label: row.label,
            start_year: row.start_year,
            end_year: row.end_year,
            archived_at: row.archived_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClosureReasonRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub label: String,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<ClosureReasonRow> for ClosureReason {
    fn from(row: ClosureReasonRow) -> Self {
        Self {
            id: ClosureReasonId(row.id),
            organization_id: OrganizationId(row.org_id),
            label: row.label,
            archived_at: row.archived_at,
            created_at: row.created_at,
        }
    }
}
