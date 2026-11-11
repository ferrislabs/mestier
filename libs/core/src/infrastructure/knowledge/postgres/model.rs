use chrono::{DateTime, Utc};
use common::{CoreError, UserId};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    EditLock, KnowledgeArticle, KnowledgeArticleId, KnowledgeFile, KnowledgeFolder,
    KnowledgeFolderId, KnowledgeKind, KnowledgeLink, KnowledgeResource, KnowledgeResourceType,
    OrganizationId,
};

#[derive(Debug, Clone)]
pub struct KnowledgeArticleRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub folder_id: Option<Uuid>,
    pub kind: String,
    pub title: String,
    pub body: Option<String>,
    pub file_key: Option<String>,
    pub mime_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub extracted_text: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub content: Option<Value>,
    pub revision: i32,
    pub lock_holder: Option<Uuid>,
    pub lock_expires_at: Option<DateTime<Utc>>,
}

impl TryFrom<KnowledgeArticleRow> for KnowledgeArticle {
    type Error = CoreError;

    fn try_from(row: KnowledgeArticleRow) -> Result<Self, Self::Error> {
        let kind = row
            .kind
            .parse::<KnowledgeKind>()
            .map_err(|()| CoreError::Internal(format!("unknown kind `{}`", row.kind)))?;
        let file = match (row.file_key, row.mime_type, row.size_bytes) {
            (Some(key), Some(mime_type), Some(size)) => Some(KnowledgeFile {
                key,
                mime_type,
                size_bytes: u64::try_from(size).map_err(|_| {
                    CoreError::Internal("negative size_bytes in knowledge_articles".into())
                })?,
            }),
            _ => None,
        };
        Ok(Self {
            id: KnowledgeArticleId(row.id),
            organization_id: OrganizationId(row.org_id),
            folder_id: row.folder_id.map(KnowledgeFolderId),
            kind,
            title: row.title,
            body: row.body,
            file,
            content: row.content,
            revision: row.revision,
            lock: row
                .lock_holder
                .zip(row.lock_expires_at)
                .map(|(holder, expires_at)| EditLock {
                    holder: UserId(holder),
                    expires_at,
                }),
            extracted_text: row.extracted_text,
            tags: row.tags,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Debug, Clone)]
pub struct KnowledgeLinkRow {
    pub article_id: Uuid,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<KnowledgeLinkRow> for KnowledgeLink {
    type Error = CoreError;

    fn try_from(row: KnowledgeLinkRow) -> Result<Self, Self::Error> {
        let resource_type = row
            .resource_type
            .parse::<KnowledgeResourceType>()
            .map_err(|()| {
                CoreError::Internal(format!("unknown resource type `{}`", row.resource_type))
            })?;
        Ok(Self {
            article_id: KnowledgeArticleId(row.article_id),
            resource: KnowledgeResource {
                resource_type,
                id: row.resource_id,
            },
            created_at: row.created_at,
        })
    }
}

#[derive(Debug, Clone)]
pub struct KnowledgeFolderRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<KnowledgeFolderRow> for KnowledgeFolder {
    fn from(row: KnowledgeFolderRow) -> Self {
        Self {
            id: KnowledgeFolderId(row.id),
            organization_id: OrganizationId(row.org_id),
            parent_id: row.parent_id.map(KnowledgeFolderId),
            name: row.name,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
