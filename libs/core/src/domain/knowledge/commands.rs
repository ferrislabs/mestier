use common::UserId;

use crate::{
    KnowledgeArticleId, KnowledgeFolder, KnowledgeFolderId, KnowledgeFolderScope,
    KnowledgeResource, OrganizationId,
};

#[derive(Debug, Clone)]
pub struct CreateKnowledgeNoteCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub title: String,
    /// Markdown, converted to blocks.
    pub body: String,
    pub tags: Vec<String>,
    /// `None` puts the document at the root.
    pub folder_id: Option<KnowledgeFolderId>,
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
    /// `None` puts the file at the root.
    pub folder_id: Option<KnowledgeFolderId>,
}

#[derive(Debug, Clone)]
pub struct ListKnowledgeCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub query: Option<String>,
    pub tags: Vec<String>,
    pub resource: Option<KnowledgeResource>,
    pub folder: Option<KnowledgeFolderScope>,
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
    pub user_id: UserId,
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

#[derive(Debug, Clone)]
pub struct CreateKnowledgeFolderCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    /// `None` creates the folder at the root.
    pub parent_id: Option<KnowledgeFolderId>,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct UpdateKnowledgeFolderCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: KnowledgeFolderId,
    /// Absent keeps the name.
    pub name: Option<String>,
    /// Absent keeps the parent; `Some(None)` moves the folder to the root.
    pub parent_id: Option<Option<KnowledgeFolderId>>,
}

#[derive(Debug, Clone)]
pub struct DeleteKnowledgeFolderCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: KnowledgeFolderId,
}

#[derive(Debug, Clone)]
pub struct ListKnowledgeFoldersCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
}

#[derive(Debug, Clone)]
pub struct BrowseKnowledgeFolderCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    /// `None` opens the root.
    pub id: Option<KnowledgeFolderId>,
}

/// A folder opened in the drive: where it is and what folders it holds. Its
/// articles come from the listing, filtered on the folder.
#[derive(Debug, Clone)]
pub struct KnowledgeFolderView {
    /// `None` at the root.
    pub folder: Option<KnowledgeFolder>,
    /// From the root down to the folder, the folder included. Empty at the root.
    pub path: Vec<KnowledgeFolder>,
    pub children: Vec<KnowledgeFolder>,
}

#[derive(Debug, Clone)]
pub struct MoveKnowledgeArticlesCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub ids: Vec<KnowledgeArticleId>,
    /// `None` moves the articles to the root.
    pub folder_id: Option<KnowledgeFolderId>,
}
