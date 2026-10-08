use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{Doe, DoeId, DoeSummary, OrganizationId, ProjectId, Revision};

#[derive(Debug, Clone)]
pub struct DoeRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub place: Option<String>,
    pub lot: Option<String>,
    pub issued_on: NaiveDate,
    pub author: Option<String>,
    pub revision: String,
    pub parties: serde_json::Value,
    pub sections: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<DoeRow> for Doe {
    type Error = CoreError;

    fn try_from(row: DoeRow) -> Result<Self, Self::Error> {
        let parties = serde_json::from_value(row.parties)
            .map_err(|e| CoreError::Internal(format!("invalid DOE parties in database: {e}")))?;
        let sections = serde_json::from_value(row.sections)
            .map_err(|e| CoreError::Internal(format!("invalid DOE sections in database: {e}")))?;
        let revision = Revision::try_from(row.revision)
            .map_err(|e| CoreError::Internal(format!("invalid DOE revision in database: {e}")))?;

        Ok(Self {
            id: DoeId(row.id),
            organization_id: OrganizationId(row.org_id),
            project_id: ProjectId(row.project_id),
            title: row.title,
            place: row.place,
            lot: row.lot,
            issued_on: row.issued_on,
            author: row.author,
            revision,
            parties,
            sections,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DoeSummaryRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub project_name: String,
    pub title: String,
    pub revision: String,
    pub issued_on: NaiveDate,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<DoeSummaryRow> for DoeSummary {
    type Error = CoreError;

    fn try_from(row: DoeSummaryRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: DoeId(row.id),
            project_id: ProjectId(row.project_id),
            project_name: row.project_name,
            title: row.title,
            revision: Revision::try_from(row.revision).map_err(|e| {
                CoreError::Internal(format!("invalid DOE revision in database: {e}"))
            })?,
            issued_on: row.issued_on,
            updated_at: row.updated_at,
        })
    }
}
