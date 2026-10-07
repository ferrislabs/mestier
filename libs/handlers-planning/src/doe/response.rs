use chrono::{DateTime, NaiveDate, Utc};
use mestier_core::{Doe, DoeId, DoeParty, DoeSection, ProjectId, Revision};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct DoeResponse {
    pub id: DoeId,
    pub project_id: ProjectId,
    pub title: String,
    pub place: Option<String>,
    pub lot: Option<String>,
    pub issued_on: NaiveDate,
    pub author: Option<String>,
    pub revision: Revision,
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
            title: doe.title,
            place: doe.place,
            lot: doe.lot,
            issued_on: doe.issued_on,
            author: doe.author,
            revision: doe.revision,
            parties: doe.parties,
            sections: doe.sections,
            created_at: doe.created_at,
            updated_at: doe.updated_at,
        }
    }
}
