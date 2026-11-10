use crate::{KnowledgeArticleId, KnowledgeResource, OrganizationId};

#[derive(Debug, Clone)]
pub struct CreateKnowledgeNoteCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UploadKnowledgeFileCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub title: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
    /// Read from the file by the caller, off the async executor.
    pub extracted_text: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ListKnowledgeCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub query: Option<String>,
    pub tags: Vec<String>,
    pub resource: Option<KnowledgeResource>,
    pub limit: u64,
    pub offset: u64,
}

#[derive(Debug, Clone)]
pub struct GetKnowledgeCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: KnowledgeArticleId,
}

#[derive(Debug, Clone)]
pub struct UpdateKnowledgeCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: KnowledgeArticleId,
    pub title: Option<String>,
    pub body: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct DeleteKnowledgeCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: KnowledgeArticleId,
}

#[derive(Debug, Clone)]
pub struct LinkKnowledgeCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: KnowledgeArticleId,
    pub resource: KnowledgeResource,
}

#[derive(Debug, Clone)]
pub struct KnowledgeArticleWithLinks {
    pub article: crate::KnowledgeArticle,
    pub links: Vec<crate::KnowledgeLink>,
}
