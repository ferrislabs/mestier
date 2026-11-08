use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    DocumentKind, DocumentName, FileKey, MemberDocument, MemberDocumentId, MemberId, OrganizationId,
};

#[derive(Debug, Clone)]
pub struct MemberDocumentRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub member_id: Uuid,
    pub name: String,
    pub kind: String,
    pub file_key: String,
    pub added_at: DateTime<Utc>,
}

fn corrupt(id: Uuid, column: &str) -> CoreError {
    CoreError::Internal(format!("member_documents {id}: invalid {column}"))
}

impl TryFrom<MemberDocumentRow> for MemberDocument {
    type Error = CoreError;

    fn try_from(row: MemberDocumentRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: MemberDocumentId(row.id),
            organization_id: OrganizationId(row.org_id),
            member_id: MemberId(row.member_id),
            name: DocumentName::try_from(row.name.as_str()).map_err(|_| corrupt(row.id, "name"))?,
            kind: row
                .kind
                .parse::<DocumentKind>()
                .map_err(|()| corrupt(row.id, "kind"))?,
            file_key: FileKey::try_from(row.file_key.as_str())
                .map_err(|_| corrupt(row.id, "file_key"))?,
            added_at: row.added_at,
        })
    }
}
