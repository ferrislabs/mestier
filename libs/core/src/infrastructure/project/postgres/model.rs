use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{CustomerContextId, CustomerId, OrganizationId, Project, ProjectId, QuoteId};

#[derive(Debug, Clone)]
pub struct ProjectRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub customer_id: Option<Uuid>,
    pub customer_context_id: Option<Uuid>,
    pub quote_id: Option<Uuid>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ProjectRow> for Project {
    fn from(row: ProjectRow) -> Self {
        Self {
            id: ProjectId(row.id),
            organization_id: OrganizationId(row.org_id),
            name: row.name,
            customer_id: row.customer_id.map(CustomerId),
            customer_context_id: row.customer_context_id.map(CustomerContextId),
            quote_id: row.quote_id.map(QuoteId),
            archived_at: row.archived_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
