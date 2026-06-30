use crate::{LegalMentionTemplateId, OrganizationId};

#[derive(Debug, Clone)]
pub struct CreateLegalMentionTemplateCommand {
	pub org_id: OrganizationId,
	pub name: String,
	pub body: String,
}

#[derive(Debug, Clone)]
pub struct UpdateLegalMentionTemplateCommand {
	pub id: LegalMentionTemplateId,
	pub name: String,
	pub body: String,
}
