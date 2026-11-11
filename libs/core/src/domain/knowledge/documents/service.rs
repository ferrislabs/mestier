use chrono::{DateTime, Utc};
use common::{CoreError, UserId};
use serde_json::Value;

use super::{
    content::{ContentError, DocumentContent},
    lock::EditLock,
    markdown::{from_markdown, render},
    ports::{
        DocumentReferenceRepository, KnowledgeDocumentRepository, LockAttempt, ReplaceAttempt,
        SaveAttempt,
    },
    references::added,
};
use crate::{
    KnowledgeArticleId, KnowledgeError, KnowledgeOutcome, OrganizationId,
    domain::knowledge::text::normalize_markdown,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocumentLockGrant {
    pub lock: EditLock,
    pub revision: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SavedDocument {
    pub revision: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentRefusal {
    Locked(EditLock),
    LockLost,
    RevisionConflict,
    Invalid(ContentError),
}

pub type DocumentOutcome<T> = Result<T, DocumentRefusal>;

pub async fn ensure_references_exist<R>(
    references: &mut R,
    organization_id: OrganizationId,
    previous: Option<&Value>,
    next: &DocumentContent,
) -> Result<(), CoreError>
where
    R: DocumentReferenceRepository,
{
    let previous = previous.and_then(|value| DocumentContent::parse(value).ok());
    for reference in added(previous.as_ref(), next) {
        if !references.exists(organization_id, reference).await? {
            return Err(CoreError::NotFound);
        }
    }
    Ok(())
}

pub fn content_from_markdown(markdown: &str) -> KnowledgeOutcome<(DocumentContent, String)> {
    let markdown = normalize_markdown(markdown)?;
    let content = from_markdown(&markdown).map_err(KnowledgeError::Content)?;
    let body = render(&content);
    Ok((content, body))
}

pub struct KnowledgeDocumentService<D>
where
    D: KnowledgeDocumentRepository,
{
    documents: D,
}

impl<D> KnowledgeDocumentService<D>
where
    D: KnowledgeDocumentRepository,
{
    pub fn new(documents: D) -> Self {
        Self { documents }
    }

    pub async fn acquire_lock(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
        user: UserId,
    ) -> Result<DocumentOutcome<DocumentLockGrant>, CoreError> {
        match self
            .documents
            .acquire_lock(organization_id, id, user)
            .await?
        {
            LockAttempt::Granted { lock, revision } => Ok(Ok(DocumentLockGrant { lock, revision })),
            LockAttempt::Held(lock) => Ok(Err(DocumentRefusal::Locked(lock))),
            LockAttempt::Missing => Err(CoreError::NotFound),
        }
    }

    pub async fn release_lock(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
        user: UserId,
    ) -> Result<(), CoreError> {
        self.documents.release_lock(organization_id, id, user).await
    }

    pub async fn save<R>(
        &mut self,
        references: &mut R,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
        user: UserId,
        revision: i32,
        content: &Value,
    ) -> Result<DocumentOutcome<SavedDocument>, CoreError>
    where
        R: DocumentReferenceRepository,
    {
        let state = self
            .documents
            .state(organization_id, id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let next = match DocumentContent::parse(content) {
            Ok(next) => next,
            Err(error) => return Ok(Err(DocumentRefusal::Invalid(error))),
        };
        ensure_references_exist(references, organization_id, Some(&state.content), &next).await?;
        let body = render(&next);
        match self
            .documents
            .save(organization_id, id, user, revision, &next, &body)
            .await?
        {
            SaveAttempt::Saved {
                revision,
                updated_at,
            } => Ok(Ok(SavedDocument {
                revision,
                updated_at,
            })),
            SaveAttempt::LockLost => Ok(Err(DocumentRefusal::LockLost)),
            SaveAttempt::RevisionConflict => Ok(Err(DocumentRefusal::RevisionConflict)),
            SaveAttempt::Missing => Err(CoreError::NotFound),
        }
    }

    pub async fn replace_from_markdown<R>(
        &mut self,
        references: &mut R,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
        user: UserId,
        markdown: &str,
    ) -> Result<KnowledgeOutcome<SavedDocument>, CoreError>
    where
        R: DocumentReferenceRepository,
    {
        let state = self
            .documents
            .state(organization_id, id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let (content, body) = match content_from_markdown(markdown) {
            Ok(written) => written,
            Err(error) => return Ok(Err(error)),
        };
        ensure_references_exist(references, organization_id, Some(&state.content), &content)
            .await?;
        match self
            .documents
            .replace(organization_id, id, user, &content, &body)
            .await?
        {
            ReplaceAttempt::Replaced {
                revision,
                updated_at,
            } => Ok(Ok(SavedDocument {
                revision,
                updated_at,
            })),
            ReplaceAttempt::Held(_) => Ok(Err(KnowledgeError::BeingEdited)),
            ReplaceAttempt::Missing => Err(CoreError::NotFound),
        }
    }
}
