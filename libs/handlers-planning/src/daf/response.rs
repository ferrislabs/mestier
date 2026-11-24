use chrono::{DateTime, NaiveDate, Utc};
use mestier_core::{
    Daf, DafAttachment, DafDossier, DafId, DafNumber, DafStatus, DafSummary, ProductId, ProjectId,
    SupplierSnapshot,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct DafDossierResponse {
    pub project_id: ProjectId,
    pub title: String,
    pub issuer: String,
    pub project_owner: Option<String>,
    pub project_manager: Option<String>,
    /// `null` while the dossier is only prefilled, not yet saved.
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<DafDossier> for DafDossierResponse {
    fn from(dossier: DafDossier) -> Self {
        Self {
            project_id: dossier.project_id,
            title: dossier.title,
            issuer: dossier.issuer,
            project_owner: dossier.project_owner,
            project_manager: dossier.project_manager,
            updated_at: dossier.updated_at,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct DafSummaryResponse {
    pub id: DafId,
    #[schema(value_type = u16)]
    pub number: DafNumber,
    pub designation: String,
    pub price_ref: Option<String>,
    pub status: DafStatus,
    pub sent_on: Option<NaiveDate>,
    pub issued_on: NaiveDate,
    pub returned_on: Option<NaiveDate>,
    pub review_note: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl From<DafSummary> for DafSummaryResponse {
    fn from(summary: DafSummary) -> Self {
        Self {
            id: summary.id,
            number: summary.number,
            designation: summary.designation.into(),
            price_ref: summary.price_ref,
            status: summary.status,
            sent_on: summary.sent_on,
            issued_on: summary.issued_on,
            returned_on: summary.returned_on,
            review_note: summary.review_note,
            updated_at: summary.updated_at,
        }
    }
}

/// The DAF page of a project: its dossier and the summary, by number.
#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct ProjectDafsResponse {
    pub dossier: DafDossierResponse,
    pub dafs: Vec<DafSummaryResponse>,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct DafResponse {
    pub id: DafId,
    pub project_id: ProjectId,
    #[schema(value_type = u16)]
    pub number: DafNumber,
    pub designation: String,
    pub price_ref: Option<String>,
    pub source_product_id: Option<ProductId>,
    pub supplier: Option<SupplierSnapshot>,
    pub installer: Option<String>,
    pub specs: Option<String>,
    pub observations: Option<String>,
    pub photo_key: Option<String>,
    pub attachments: Vec<DafAttachment>,
    /// `null` prints the dossier's.
    pub project_owner: Option<String>,
    /// `null` prints the dossier's.
    pub project_manager: Option<String>,
    pub status: DafStatus,
    pub sent_on: Option<NaiveDate>,
    pub issued_on: NaiveDate,
    pub returned_on: Option<NaiveDate>,
    pub review_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Daf> for DafResponse {
    fn from(daf: Daf) -> Self {
        Self {
            id: daf.id,
            project_id: daf.project_id,
            number: daf.number,
            designation: daf.designation.into(),
            price_ref: daf.price_ref,
            source_product_id: daf.source_product_id,
            supplier: daf.supplier,
            installer: daf.installer,
            specs: daf.specs,
            observations: daf.observations,
            photo_key: daf.photo_key,
            attachments: daf.attachments,
            project_owner: daf.project_owner,
            project_manager: daf.project_manager,
            status: daf.status,
            sent_on: daf.sent_on,
            issued_on: daf.issued_on,
            returned_on: daf.returned_on,
            review_note: daf.review_note,
            created_at: daf.created_at,
            updated_at: daf.updated_at,
        }
    }
}
