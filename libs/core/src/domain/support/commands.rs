use authz::Subject;

use crate::{
    Badge, CustomFields, MemberId, OrganizationId, TicketId, TicketStatus,
    domain::support::ports::TicketFilter,
};

pub struct CreateTicketCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub subject: String,
    pub body: String,
    pub custom_fields: CustomFields,
    pub external_id: Option<String>,
}

pub struct GetTicketCommand {
    pub organization_id: OrganizationId,
    pub ticket_id: TicketId,
}

pub struct ListTicketsCommand {
    pub organization_id: OrganizationId,
    pub filter: TicketFilter,
    pub offset: u64,
    pub limit: u64,
}

pub struct TransitionTicketCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub ticket_id: TicketId,
    pub status: TicketStatus,
}

pub struct AssignTicketCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub ticket_id: TicketId,
    pub assignee: Option<MemberId>,
}

pub struct AddBadgeCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub ticket_id: TicketId,
    pub badge: Badge,
}

pub struct RemoveBadgeCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub ticket_id: TicketId,
    pub label: String,
}
