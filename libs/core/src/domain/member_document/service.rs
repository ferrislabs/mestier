use authz::{Authorizer, Resource, Subject};
use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use super::{
    MemberDocument, MemberDocumentId, MemberDocumentOutcome,
    commands::{AddMemberDocumentCommand, DeleteMemberDocumentCommand, ListMemberDocumentsCommand},
    ports::MemberDocumentRepository,
};
use crate::{
    MemberId, OrganizationId,
    application::policy,
    domain::{member::ports::MemberRepository, role::ports::RoleRepository},
};

pub struct MemberDocumentService<D, M, Ro, A>
where
    D: MemberDocumentRepository,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
{
    documents: D,
    members: M,
    roles: Ro,
    authz: A,
}

impl<D, M, Ro, A> MemberDocumentService<D, M, Ro, A>
where
    D: MemberDocumentRepository,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
{
    pub fn new(documents: D, members: M, roles: Ro, authz: A) -> Self {
        Self {
            documents,
            members,
            roles,
            authz,
        }
    }

    pub async fn list(
        &mut self,
        command: ListMemberDocumentsCommand,
    ) -> Result<Vec<MemberDocument>, CoreError> {
        self.authorize(command.actor, command.organization_id)
            .await?;
        self.documents
            .list_by_organization(command.organization_id)
            .await
    }

    pub async fn add(
        &mut self,
        command: AddMemberDocumentCommand,
    ) -> Result<MemberDocumentOutcome<MemberDocument>, CoreError> {
        self.authorize(command.actor, command.organization_id)
            .await?;
        self.require_member(command.organization_id, command.member_id)
            .await?;
        let document = match MemberDocument::new(
            MemberDocumentId(generate_uuid_v7()),
            command.organization_id,
            command.member_id,
            &command.name,
            command.kind,
            &command.file_key,
            Utc::now(),
        ) {
            Ok(document) => document,
            Err(refusal) => return Ok(Err(refusal)),
        };
        self.documents.insert(&document).await.map(Ok)
    }

    pub async fn delete(
        &mut self,
        command: DeleteMemberDocumentCommand,
    ) -> Result<MemberDocument, CoreError> {
        self.authorize(command.actor, command.organization_id)
            .await?;
        self.require_member(command.organization_id, command.member_id)
            .await?;
        let document = self
            .documents
            .find(command.organization_id, command.document_id)
            .await?
            .filter(|document| document.member_id == command.member_id)
            .ok_or(CoreError::NotFound)?;
        self.documents
            .delete(command.organization_id, command.document_id)
            .await?;
        Ok(document)
    }

    async fn authorize(
        &mut self,
        actor: Subject,
        organization_id: OrganizationId,
    ) -> Result<(), CoreError> {
        let actor = policy::enrich_for_organization(
            actor,
            organization_id,
            &mut self.members,
            &mut self.roles,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "member.manage",
            Resource::new("organization", organization_id.0.to_string()),
        )
        .await
    }

    async fn require_member(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
    ) -> Result<(), CoreError> {
        self.members
            .find_by_id(member_id)
            .await?
            .filter(|member| member.organization_id == organization_id)
            .map(|_| ())
            .ok_or(CoreError::NotFound)
    }
}
