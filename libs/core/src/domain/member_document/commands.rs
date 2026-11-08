use authz::Subject;

use super::{DocumentKind, MemberDocumentId};
use crate::{MemberId, OrganizationId};

#[derive(Debug, Clone)]
pub struct ListMemberDocumentsCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
}

#[derive(Debug, Clone)]
pub struct AddMemberDocumentCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub name: String,
    pub kind: DocumentKind,
    pub file_key: String,
}

#[derive(Debug, Clone)]
pub struct DeleteMemberDocumentCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub document_id: MemberDocumentId,
}
