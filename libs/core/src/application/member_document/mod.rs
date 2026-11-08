use common::CoreError;
use mestier_macros::transactional;

use crate::{
    MemberDocument, MemberDocumentOutcome,
    application::MestierUseCase,
    domain::member_document::{
        MemberDocumentService,
        commands::{
            AddMemberDocumentCommand, DeleteMemberDocumentCommand, ListMemberDocumentsCommand,
        },
    },
};

impl MestierUseCase {
    #[transactional(member_document, member, role, authz)]
    pub async fn list_member_documents(
        &self,
        command: ListMemberDocumentsCommand,
    ) -> Result<Vec<MemberDocument>, CoreError> {
        let mut service = MemberDocumentService::new(
            member_document_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.list(command).await
    }

    #[transactional(member_document, member, role, authz)]
    pub async fn add_member_document(
        &self,
        command: AddMemberDocumentCommand,
    ) -> Result<MemberDocumentOutcome<MemberDocument>, CoreError> {
        let mut service = MemberDocumentService::new(
            member_document_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.add(command).await
    }

    #[transactional(member_document, member, role, authz)]
    pub async fn delete_member_document(
        &self,
        command: DeleteMemberDocumentCommand,
    ) -> Result<MemberDocument, CoreError> {
        let mut service = MemberDocumentService::new(
            member_document_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.delete(command).await
    }
}
