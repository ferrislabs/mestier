use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    Daf, DafDesignation, DafDossier, DafId, DafNumber, DafStatus, DafSummary, OrganizationId,
    ProductId, ProjectId,
};

#[derive(Debug, Clone)]
pub struct DafDossierRow {
    pub project_id: Uuid,
    pub org_id: Uuid,
    pub title: String,
    pub issuer: String,
    pub project_owner: Option<String>,
    pub project_manager: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl From<DafDossierRow> for DafDossier {
    fn from(row: DafDossierRow) -> Self {
        Self {
            organization_id: OrganizationId(row.org_id),
            project_id: ProjectId(row.project_id),
            title: row.title,
            issuer: row.issuer,
            project_owner: row.project_owner,
            project_manager: row.project_manager,
            updated_at: Some(row.updated_at),
        }
    }
}

fn number(value: i32) -> Result<DafNumber, CoreError> {
    u16::try_from(value)
        .map_err(|e| CoreError::Internal(format!("invalid DAF number in database: {e}")))
        .and_then(|value| {
            DafNumber::try_from(value)
                .map_err(|e| CoreError::Internal(format!("invalid DAF number in database: {e}")))
        })
}

fn designation(value: String) -> Result<DafDesignation, CoreError> {
    DafDesignation::try_from(value)
        .map_err(|e| CoreError::Internal(format!("invalid DAF designation in database: {e}")))
}

#[derive(Debug, Clone)]
pub struct DafRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub project_id: Uuid,
    pub number: i32,
    pub designation: String,
    pub price_ref: Option<String>,
    pub source_product_id: Option<Uuid>,
    pub supplier: Option<serde_json::Value>,
    pub installer: Option<String>,
    pub specs: Option<String>,
    pub observations: Option<String>,
    pub photo_key: Option<String>,
    pub attachments: serde_json::Value,
    pub project_owner: Option<String>,
    pub project_manager: Option<String>,
    pub status: String,
    pub sent_on: Option<NaiveDate>,
    pub issued_on: NaiveDate,
    pub returned_on: Option<NaiveDate>,
    pub review_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<DafRow> for Daf {
    type Error = CoreError;

    fn try_from(row: DafRow) -> Result<Self, Self::Error> {
        let supplier = row
            .supplier
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| CoreError::Internal(format!("invalid DAF supplier in database: {e}")))?;
        let attachments = serde_json::from_value(row.attachments).map_err(|e| {
            CoreError::Internal(format!("invalid DAF attachments in database: {e}"))
        })?;

        Ok(Self {
            id: DafId(row.id),
            organization_id: OrganizationId(row.org_id),
            project_id: ProjectId(row.project_id),
            number: number(row.number)?,
            designation: designation(row.designation)?,
            price_ref: row.price_ref,
            source_product_id: row.source_product_id.map(ProductId),
            supplier,
            installer: row.installer,
            specs: row.specs,
            observations: row.observations,
            photo_key: row.photo_key,
            attachments,
            project_owner: row.project_owner,
            project_manager: row.project_manager,
            status: row.status.parse::<DafStatus>()?,
            sent_on: row.sent_on,
            issued_on: row.issued_on,
            returned_on: row.returned_on,
            review_note: row.review_note,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DafSummaryRow {
    pub id: Uuid,
    pub number: i32,
    pub designation: String,
    pub price_ref: Option<String>,
    pub status: String,
    pub sent_on: Option<NaiveDate>,
    pub issued_on: NaiveDate,
    pub returned_on: Option<NaiveDate>,
    pub review_note: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<DafSummaryRow> for DafSummary {
    type Error = CoreError;

    fn try_from(row: DafSummaryRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: DafId(row.id),
            number: number(row.number)?,
            designation: designation(row.designation)?,
            price_ref: row.price_ref,
            status: row.status.parse::<DafStatus>()?,
            sent_on: row.sent_on,
            issued_on: row.issued_on,
            returned_on: row.returned_on,
            review_note: row.review_note,
            updated_at: row.updated_at,
        })
    }
}
