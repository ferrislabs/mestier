use common::UserId;
use serde_json::Value;

use crate::{KnowledgeArticleId, KnowledgeFolderId, OrganizationId};

#[derive(Debug, Clone)]
pub struct CreateKnowledgeDocumentCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub title: String,
    pub tags: Vec<String>,
    pub folder_id: Option<KnowledgeFolderId>,
}

#[derive(Debug, Clone)]
pub struct DocumentLockCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub id: KnowledgeArticleId,
}

#[derive(Debug, Clone)]
pub struct SaveDocumentContentCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub id: KnowledgeArticleId,
    pub content: Value,
    pub revision: i32,
}
