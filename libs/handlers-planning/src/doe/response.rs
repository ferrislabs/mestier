use chrono::{DateTime, NaiveDate, Utc};
use mestier_core::{Doe, DoeId, DoeModelId, DoeParty, DoeSection, DoeSummary, ProjectId, Revision};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct DoeResponse {
    pub id: DoeId,
    pub project_id: ProjectId,
    /// The model the DOE was made from, whose style it is printed in.
    pub model_id: Option<DoeModelId>,
    pub title: String,
    pub place: Option<String>,
    pub lot: Option<String>,
    pub issued_on: NaiveDate,
    pub author: Option<String>,
    pub revision: Revision,
    pub cover_photo_key: Option<String>,
    pub parties: Vec<DoeParty>,
    pub sections: Vec<DoeSection>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Doe> for DoeResponse {
    fn from(doe: Doe) -> Self {
        Self {
            id: doe.id,
            project_id: doe.project_id,
            model_id: doe.model_id,
            title: doe.title,
            place: doe.place,
            lot: doe.lot,
            issued_on: doe.issued_on,
            author: doe.author,
            revision: doe.revision,
            cover_photo_key: doe.cover_photo_key,
            parties: doe.parties,
            sections: doe.sections,
            created_at: doe.created_at,
            updated_at: doe.updated_at,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct DoeSummaryResponse {
    pub id: DoeId,
    pub project_id: ProjectId,
    pub project_name: String,
    pub title: String,
    pub revision: Revision,
    pub issued_on: NaiveDate,
    pub updated_at: DateTime<Utc>,
}

impl From<DoeSummary> for DoeSummaryResponse {
    fn from(summary: DoeSummary) -> Self {
        Self {
            id: summary.id,
            project_id: summary.project_id,
            project_name: summary.project_name,
            title: summary.title,
            revision: summary.revision,
            issued_on: summary.issued_on,
            updated_at: summary.updated_at,
        }
    }
}
