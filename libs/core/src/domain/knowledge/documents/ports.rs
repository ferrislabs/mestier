use chrono::{DateTime, Utc};
use common::{CoreError, UserId};
use serde_json::Value;

use super::{content::DocumentContent, lock::EditLock, references::DocumentReference};
use crate::{KnowledgeArticleId, OrganizationId};

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentState {
    pub content: Value,
    pub revision: i32,
    pub lock: Option<EditLock>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockAttempt {
    Granted { lock: EditLock, revision: i32 },
    Held(EditLock),
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveAttempt {
    Saved {
        revision: i32,
        updated_at: DateTime<Utc>,
    },
    LockLost,
    RevisionConflict,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplaceAttempt {
    Replaced {
        revision: i32,
        updated_at: DateTime<Utc>,
    },
    Held(EditLock),
    Missing,
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait KnowledgeDocumentRepository: Send {
    fn state(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
    ) -> impl Future<Output = Result<Option<DocumentState>, CoreError>> + Send;
    fn acquire_lock(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
        user: UserId,
    ) -> impl Future<Output = Result<LockAttempt, CoreError>> + Send;
    fn release_lock(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
        user: UserId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn save(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
        user: UserId,
        revision: i32,
        content: &DocumentContent,
        body: &str,
    ) -> impl Future<Output = Result<SaveAttempt, CoreError>> + Send;
    fn replace(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
        user: UserId,
        content: &DocumentContent,
        body: &str,
    ) -> impl Future<Output = Result<ReplaceAttempt, CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait DocumentReferenceRepository: Send {
    fn exists(
        &mut self,
        organization_id: OrganizationId,
        reference: DocumentReference,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}
