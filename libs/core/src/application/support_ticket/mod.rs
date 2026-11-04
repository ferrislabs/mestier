use authz::{Resource, Subject};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    AddBadgeCommand, AssignTicketCommand, CreateTicketCommand, CreatedTicket, GetTicketCommand,
    ListTicketsCommand, OrganizationId, RemoveBadgeCommand, Ticket, TicketPage,
    TransitionTicketCommand,
    application::{MestierAuthorizer, MestierUseCase, policy},
    domain::{
        member::ports::MemberRepository, role::ports::RoleRepository,
        support::ticket_service::TicketService,
    },
};

#[cfg(test)]
pub(crate) mod tests;

pub(crate) async fn authorize<M, R>(
    authorizer: &MestierAuthorizer,
    actor: Subject,
    organization_id: OrganizationId,
    members: &mut M,
    roles: &mut R,
) -> Result<Subject, CoreError>
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
    Ok(actor)
}

impl MestierUseCase {
    #[transactional(support_ticket, member, role, authz, emitter)]
    pub async fn create_support_ticket(
        &self,
        mut command: CreateTicketCommand,
    ) -> Result<CreatedTicket, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        command.actor = authorize(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        TicketService::new(support_ticket_repository, emitter)
            .create(command)
            .await
    }

    #[transactional(support_ticket, emitter)]
    pub async fn get_support_ticket(&self, command: GetTicketCommand) -> Result<Ticket, CoreError> {
        TicketService::new(support_ticket_repository, emitter)
            .get(command.organization_id, command.ticket_id)
            .await
    }

    #[transactional(support_ticket, emitter)]
    pub async fn list_support_tickets(
        &self,
        command: ListTicketsCommand,
    ) -> Result<TicketPage, CoreError> {
        TicketService::new(support_ticket_repository, emitter)
            .list(command)
            .await
    }

    #[transactional(support_ticket, member, role, authz, emitter)]
    pub async fn transition_support_ticket(
        &self,
        mut command: TransitionTicketCommand,
    ) -> Result<Ticket, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        command.actor = authorize(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        TicketService::new(support_ticket_repository, emitter)
            .transition(command)
            .await
    }

    #[transactional(support_ticket, member, role, authz, emitter)]
    pub async fn assign_support_ticket(
        &self,
        mut command: AssignTicketCommand,
    ) -> Result<Ticket, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        command.actor = authorize(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        if let Some(assignee) = command.assignee {
            member_repository
                .find_by_id(assignee)
                .await?
                .filter(|member| member.organization_id == command.organization_id)
                .ok_or(CoreError::NotFound)?;
        }

        TicketService::new(support_ticket_repository, emitter)
            .assign(command)
            .await
    }

    #[transactional(support_ticket, member, role, authz, emitter)]
    pub async fn add_support_ticket_badge(
        &self,
        mut command: AddBadgeCommand,
    ) -> Result<Ticket, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        command.actor = authorize(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        TicketService::new(support_ticket_repository, emitter)
            .add_badge(command)
            .await
    }

    #[transactional(support_ticket, member, role, authz, emitter)]
    pub async fn remove_support_ticket_badge(
        &self,
        mut command: RemoveBadgeCommand,
    ) -> Result<Ticket, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        command.actor = authorize(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        TicketService::new(support_ticket_repository, emitter)
            .remove_badge(command)
            .await
    }
}
