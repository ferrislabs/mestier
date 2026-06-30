use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{LegalMentionTemplate, LegalMentionTemplateId, OrganizationId};

#[derive(Debug, Clone)]
pub struct LegalMentionTemplateRow {
	pub id: Uuid,
	pub org_id: Uuid,
	pub name: String,
	pub body: String,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
	pub deleted_at: Option<DateTime<Utc>>,
}

impl TryFrom<LegalMentionTemplateRow> for LegalMentionTemplate {
	type Error = CoreError;

	fn try_from(row: LegalMentionTemplateRow) -> Result<Self, Self::Error> {
		Ok(Self {
			id: LegalMentionTemplateId(row.id),
			org_id: OrganizationId(row.org_id),
			name: row.name,
			body: row.body,
			created_at: row.created_at,
			updated_at: row.updated_at,
			deleted_at: row.deleted_at,
		})
	}
}
