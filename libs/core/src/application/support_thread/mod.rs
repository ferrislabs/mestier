use authz::{Resource, Subject};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    AddEntryCommand, AttachFileCommand, Attachment, AttachmentId, DraftCommand, EditDraftCommand,
    ListThreadCommand, LocatedAttachment, MemberId, OrganizationId, PostClientMessageCommand,
    TicketEntry, TicketEntryId,
    application::{
        MestierAuthorizer, MestierUseCase,
        policy::{self, SUBJECT_MEMBER_ID_KEY},
    },
    domain::{
        member::ports::MemberRepository,
        role::ports::RoleRepository,
        support::thread_service::{NewAttachment, ThreadService},
    },
};

#[cfg(test)]
pub(crate) mod tests;

async fn authorize<M, R>(
    authorizer: &MestierAuthorizer,
    actor: Subject,
    organization_id: OrganizationId,
    members: &mut M,
    roles: &mut R,
) -> Result<MemberId, CoreError>
where
    M: MemberRepository,
    R: RoleRepository,
{
    let actor = policy::enrich_for_organization(actor, organization_id, members, roles).await?;
    policy::require(
        authorizer,
        &actor,
        "support.manage",
        Resource::new("organization", organization_id.0.to_string()),
    )
    .await?;
    actor
        .properties
        .get(SUBJECT_MEMBER_ID_KEY)
        .and_then(|value| value.as_str())
        .and_then(|raw| raw.parse().ok())
        .map(MemberId)
        .ok_or(CoreError::Forbidden {
            reason: Some("this action needs an organization member".to_owned()),
        })
}

impl MestierUseCase {
    #[transactional(support_ticket, support_ticket_entry, member, role, authz, emitter)]
    pub async fn add_support_internal_note(
        &self,
        command: AddEntryCommand,
    ) -> Result<TicketEntry, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let author = authorize(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        ThreadService::new(
            support_ticket_repository,
            support_ticket_entry_repository,
            emitter,
        )
        .add_note(
            command.organization_id,
            command.ticket_id,
            author,
            &command.body,
        )
        .await
    }

    #[transactional(support_ticket, support_ticket_entry, member, role, authz, emitter)]
    pub async fn add_support_automation_note(
        &self,
        command: AddEntryCommand,
    ) -> Result<TicketEntry, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        crate::application::support_ticket::authorize(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        ThreadService::new(
            support_ticket_repository,
            support_ticket_entry_repository,
            emitter,
        )
        .add_automation_note(command.organization_id, command.ticket_id, &command.body)
        .await
    }

    #[transactional(support_ticket, support_ticket_entry, member, role, authz, emitter)]
    pub async fn save_support_draft(
        &self,
        command: AddEntryCommand,
    ) -> Result<TicketEntry, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let author = authorize(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        ThreadService::new(
            support_ticket_repository,
            support_ticket_entry_repository,
            emitter,
        )
        .save_draft(
            command.organization_id,
            command.ticket_id,
            author,
            &command.body,
        )
        .await
    }

    #[transactional(support_ticket, support_ticket_entry, member, role, authz, emitter)]
    pub async fn send_support_reply(
        &self,
        command: AddEntryCommand,
    ) -> Result<TicketEntry, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let author = authorize(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        ThreadService::new(
            support_ticket_repository,
            support_ticket_entry_repository,
            emitter,
        )
        .send_reply(
            command.organization_id,
            command.ticket_id,
            author,
            &command.body,
        )
        .await
    }

    #[transactional(support_ticket, support_ticket_entry, member, role, authz, emitter)]
    pub async fn edit_support_draft(
        &self,
        command: EditDraftCommand,
    ) -> Result<TicketEntry, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        authorize(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        ThreadService::new(
            support_ticket_repository,
            support_ticket_entry_repository,
            emitter,
        )
        .edit_draft(command.organization_id, command.entry_id, &command.body)
        .await
    }

    #[transactional(support_ticket, support_ticket_entry, member, role, authz, emitter)]
    pub async fn delete_support_draft(&self, command: DraftCommand) -> Result<(), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        authorize(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        ThreadService::new(
            support_ticket_repository,
            support_ticket_entry_repository,
            emitter,
        )
        .delete_draft(command.organization_id, command.entry_id)
        .await
    }

    #[transactional(support_ticket, support_ticket_entry, member, role, authz, emitter)]
    pub async fn send_support_draft(
        &self,
        command: DraftCommand,
    ) -> Result<TicketEntry, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let sender = authorize(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        ThreadService::new(
            support_ticket_repository,
            support_ticket_entry_repository,
            emitter,
        )
        .send_draft(command.organization_id, command.entry_id, sender)
        .await
    }

    #[transactional(support_ticket, support_ticket_entry, member, role, authz, emitter)]
    pub async fn attach_to_support_entry(
        &self,
        command: AttachFileCommand,
    ) -> Result<Attachment, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        authorize(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        ThreadService::new(
            support_ticket_repository,
            support_ticket_entry_repository,
            emitter,
        )
        .attach_member_file(
            command.organization_id,
            command.entry_id,
            NewAttachment {
                file_key: command.file_key,
                file_name: command.file_name,
                mime_type: command.mime_type,
                size_bytes: command.size_bytes,
            },
        )
        .await
    }

    #[transactional(support_ticket, support_ticket_entry, emitter)]
    pub async fn list_support_thread(
        &self,
        command: ListThreadCommand,
    ) -> Result<Vec<TicketEntry>, CoreError> {
        ThreadService::new(
            support_ticket_repository,
            support_ticket_entry_repository,
            emitter,
        )
        .list(
            command.organization_id,
            command.ticket_id,
            command.client_visible_only,
        )
        .await
    }

    #[transactional(support_ticket, support_ticket_entry, emitter)]
    pub async fn post_support_client_message(
        &self,
        command: PostClientMessageCommand,
    ) -> Result<TicketEntry, CoreError> {
        ThreadService::new(
            support_ticket_repository,
            support_ticket_entry_repository,
            emitter,
        )
        .post_client_message(command.organization_id, command.ticket_id, &command.body)
        .await
    }

    #[transactional(support_ticket, support_ticket_entry, emitter)]
    pub async fn attach_to_support_client_message(
        &self,
        organization_id: OrganizationId,
        entry_id: TicketEntryId,
        attachment: NewAttachment,
    ) -> Result<Attachment, CoreError> {
        ThreadService::new(
            support_ticket_repository,
            support_ticket_entry_repository,
            emitter,
        )
        .attach_client_file(organization_id, entry_id, attachment)
        .await
    }

    #[transactional(support_ticket, support_ticket_entry, emitter)]
    pub async fn get_support_attachment(
        &self,
        organization_id: OrganizationId,
        attachment_id: AttachmentId,
    ) -> Result<LocatedAttachment, CoreError> {
        ThreadService::new(
            support_ticket_repository,
            support_ticket_entry_repository,
            emitter,
        )
        .get_attachment(organization_id, attachment_id)
        .await
    }
}
