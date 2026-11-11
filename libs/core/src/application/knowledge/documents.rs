use common::CoreError;
use mestier_macros::transactional;

use super::require;
use crate::{
    CreateKnowledgeDocumentCommand, DocumentLockCommand, DocumentLockGrant, DocumentOutcome,
    KnowledgeArticle, KnowledgeOutcome, SaveDocumentContentCommand, SavedDocument,
    application::MestierUseCase,
    domain::knowledge::{
        documents::{content::DocumentContent, service::KnowledgeDocumentService},
        folders::KnowledgeFolderService,
        service::KnowledgeService,
    },
};

impl MestierUseCase {
    #[transactional(knowledge_article, knowledge_folder, member, role, authz)]
    pub async fn create_knowledge_document(
        &self,
        command: CreateKnowledgeDocumentCommand,
    ) -> Result<KnowledgeOutcome<KnowledgeArticle>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeFolderService::new(knowledge_folder_repository)
            .require(command.organization_id, command.folder_id)
            .await?;
        let content = DocumentContent::empty();
        KnowledgeService::new(knowledge_article_repository)
            .create_document(
                command.organization_id,
                command.folder_id,
                &command.title,
                &command.tags,
                content,
                String::new(),
            )
            .await
    }

    #[transactional(knowledge_document, member, role, authz)]
    pub async fn acquire_document_lock(
        &self,
        command: DocumentLockCommand,
    ) -> Result<DocumentOutcome<DocumentLockGrant>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeDocumentService::new(knowledge_document_repository)
            .acquire_lock(command.organization_id, command.id, command.user_id)
            .await
    }

    #[transactional(knowledge_document, member, role, authz)]
    pub async fn release_document_lock(
        &self,
        command: DocumentLockCommand,
    ) -> Result<(), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeDocumentService::new(knowledge_document_repository)
            .release_lock(command.organization_id, command.id, command.user_id)
            .await
    }

    #[transactional(knowledge_document, document_reference, member, role, authz)]
    pub async fn save_document_content(
        &self,
        command: SaveDocumentContentCommand,
    ) -> Result<DocumentOutcome<SavedDocument>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let mut document_reference_repository = document_reference_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeDocumentService::new(knowledge_document_repository)
            .save(
                &mut document_reference_repository,
                command.organization_id,
                command.id,
                command.user_id,
                command.revision,
                &command.content,
            )
            .await
    }
}
