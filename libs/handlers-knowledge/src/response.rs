use chrono::{DateTime, Utc};
use mestier_core::{
    EditLock, KnowledgeArticle, KnowledgeArticleId, KnowledgeFolder, KnowledgeFolderId,
    KnowledgeFolderView, KnowledgeKind, KnowledgeLink, KnowledgeResourceType,
};
use serde::Serialize;
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct KnowledgeFileResponse {
    pub mime_type: String,
    pub size_bytes: u64,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct KnowledgeLockResponse {
    pub holder_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

impl From<EditLock> for KnowledgeLockResponse {
    fn from(lock: EditLock) -> Self {
        Self {
            holder_id: lock.holder.0,
            expires_at: lock.expires_at,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct KnowledgeLinkResponse {
    pub resource_type: KnowledgeResourceType,
    pub resource_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<KnowledgeLink> for KnowledgeLinkResponse {
    fn from(link: KnowledgeLink) -> Self {
        Self {
            resource_type: link.resource.resource_type,
            resource_id: link.resource.id,
            created_at: link.created_at,
        }
    }
}

/// An article. The storage key is never exposed: a file is read through the
/// download route. The text read from a file is only in the single-article
/// response, never in a list.
#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct KnowledgeArticleResponse {
    pub id: KnowledgeArticleId,
    /// Absent when the article sits at the root of the drive.
    pub folder_id: Option<KnowledgeFolderId>,
    pub kind: KnowledgeKind,
    pub title: String,
    /// The Markdown of a document, written by the server on every save. Absent
    /// for a file.
    pub body: Option<String>,
    /// Absent for a document.
    pub file: Option<KnowledgeFileResponse>,
    /// The blocks of a document, in the Mestier block format (version 1).
    /// Only on the single-article response.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<Object>)]
    pub content: Option<Value>,
    /// Increases by one on every save of a document. Absent for a file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<i32>,
    /// Who holds the edit lock of a document and until when. Null when it is
    /// free, expired, or for a file.
    pub lock: Option<KnowledgeLockResponse>,
    /// Text read from the file, when it holds any. Only on the single-article
    /// response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extracted_text: Option<String>,
    /// Only on the single-article response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<KnowledgeLinkResponse>>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<KnowledgeArticle> for KnowledgeArticleResponse {
    fn from(article: KnowledgeArticle) -> Self {
        let is_document = article.kind == KnowledgeKind::Document;
        let now = Utc::now();
        Self {
            id: article.id,
            folder_id: article.folder_id,
            kind: article.kind,
            title: article.title,
            body: article.body,
            file: article.file.map(|file| KnowledgeFileResponse {
                mime_type: file.mime_type,
                size_bytes: file.size_bytes,
            }),
            content: article.content,
            revision: is_document.then_some(article.revision),
            lock: article
                .lock
                .filter(|lock| lock.is_active(now))
                .map(Into::into),
            extracted_text: article.extracted_text,
            links: None,
            tags: article.tags,
            created_at: article.created_at,
            updated_at: article.updated_at,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct KnowledgeFolderResponse {
    pub id: KnowledgeFolderId,
    /// Absent at the root.
    pub parent_id: Option<KnowledgeFolderId>,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<KnowledgeFolder> for KnowledgeFolderResponse {
    fn from(folder: KnowledgeFolder) -> Self {
        Self {
            id: folder.id,
            parent_id: folder.parent_id,
            name: folder.name,
            created_at: folder.created_at,
            updated_at: folder.updated_at,
        }
    }
}

/// An opened folder. Its articles come from the article listing filtered on
/// the folder.
#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct KnowledgeFolderViewResponse {
    /// Absent at the root.
    pub folder: Option<KnowledgeFolderResponse>,
    /// From the root down to the folder, the folder included. Empty at the root.
    pub path: Vec<KnowledgeFolderResponse>,
    /// The folders directly inside, by name.
    pub children: Vec<KnowledgeFolderResponse>,
}

impl From<KnowledgeFolderView> for KnowledgeFolderViewResponse {
    fn from(view: KnowledgeFolderView) -> Self {
        Self {
            folder: view.folder.map(Into::into),
            path: view.path.into_iter().map(Into::into).collect(),
            children: view.children.into_iter().map(Into::into).collect(),
        }
    }
}
