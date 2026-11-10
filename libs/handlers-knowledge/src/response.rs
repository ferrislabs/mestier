use chrono::{DateTime, Utc};
use mestier_core::{
    KnowledgeArticle, KnowledgeArticleId, KnowledgeKind, KnowledgeLink, KnowledgeResourceType,
};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct KnowledgeFileResponse {
    pub mime_type: String,
    pub size_bytes: u64,
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
    pub kind: KnowledgeKind,
    pub title: String,
    /// The text of a note. Absent for a file.
    pub body: Option<String>,
    /// Absent for a note.
    pub file: Option<KnowledgeFileResponse>,
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
        Self {
            id: article.id,
            kind: article.kind,
            title: article.title,
            body: article.body,
            file: article.file.map(|file| KnowledgeFileResponse {
                mime_type: file.mime_type,
                size_bytes: file.size_bytes,
            }),
            extracted_text: article.extracted_text,
            links: None,
            tags: article.tags,
            created_at: article.created_at,
            updated_at: article.updated_at,
        }
    }
}
