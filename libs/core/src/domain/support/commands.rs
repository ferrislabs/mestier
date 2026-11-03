use authz::Subject;

use crate::{
    Badge, CustomFields, MemberId, OrganizationId, TicketEntryId, TicketId, TicketStatus,
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

pub struct AddEntryCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub ticket_id: TicketId,
    pub body: String,
}

pub struct EditDraftCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub entry_id: TicketEntryId,
    pub body: String,
}

pub struct DraftCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub entry_id: TicketEntryId,
}

pub struct ListThreadCommand {
    pub organization_id: OrganizationId,
    pub ticket_id: TicketId,
    pub client_visible_only: bool,
}

pub struct AttachFileCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub entry_id: TicketEntryId,
    pub file_key: String,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: u64,
}

pub struct PostClientMessageCommand {
    pub organization_id: OrganizationId,
    pub ticket_id: TicketId,
    pub body: String,
}
