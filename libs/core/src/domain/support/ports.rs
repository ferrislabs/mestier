use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{
    Attachment, AttachmentId, DisplayFields, LocatedAttachment, MemberId, OrganizationId,
    SupportApiKey, SupportApiKeyId, Ticket, TicketEntry, TicketEntryId, TicketId, TicketNumber,
    TicketStatus,
};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TicketFilter {
    pub statuses: Vec<TicketStatus>,
    pub assignee: Option<MemberId>,
    pub badge: Option<String>,
    pub custom_field: Option<(String, String)>,
}

pub type LastAssignment = (MemberId, Option<DateTime<Utc>>);

#[derive(Debug, Clone, PartialEq)]
pub struct TicketPage {
    pub items: Vec<Ticket>,
    pub total: u64,
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait TicketRepository: Send {
    fn create(&mut self, ticket: &Ticket)
    -> impl Future<Output = Result<Ticket, CoreError>> + Send;

    fn find(
        &mut self,
        organization_id: OrganizationId,
        id: TicketId,
    ) -> impl Future<Output = Result<Option<Ticket>, CoreError>> + Send;

    fn find_by_external_id(
        &mut self,
        organization_id: OrganizationId,
        external_id: &str,
    ) -> impl Future<Output = Result<Option<Ticket>, CoreError>> + Send;

    fn list(
        &mut self,
        organization_id: OrganizationId,
        filter: &TicketFilter,
        offset: u64,
        limit: u64,
    ) -> impl Future<Output = Result<TicketPage, CoreError>> + Send;

    fn update(&mut self, ticket: &Ticket)
    -> impl Future<Output = Result<Ticket, CoreError>> + Send;

    fn next_number(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<TicketNumber, CoreError>> + Send;

    fn last_assigned_at(
        &mut self,
        organization_id: OrganizationId,
        members: &[MemberId],
    ) -> impl Future<Output = Result<Vec<LastAssignment>, CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait TicketEntryRepository: Send {
    fn create(
        &mut self,
        entry: &TicketEntry,
    ) -> impl Future<Output = Result<TicketEntry, CoreError>> + Send;

    fn find(
        &mut self,
        organization_id: OrganizationId,
        id: TicketEntryId,
    ) -> impl Future<Output = Result<Option<TicketEntry>, CoreError>> + Send;

    fn list_for_ticket(
        &mut self,
        organization_id: OrganizationId,
        ticket_id: TicketId,
        client_visible_only: bool,
    ) -> impl Future<Output = Result<Vec<TicketEntry>, CoreError>> + Send;

    fn update(
        &mut self,
        entry: &TicketEntry,
    ) -> impl Future<Output = Result<TicketEntry, CoreError>> + Send;

    fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: TicketEntryId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn add_attachment(
        &mut self,
        attachment: &Attachment,
    ) -> impl Future<Output = Result<Attachment, CoreError>> + Send;

    fn find_attachment(
        &mut self,
        organization_id: OrganizationId,
        id: AttachmentId,
    ) -> impl Future<Output = Result<Option<LocatedAttachment>, CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait SupportSettingsRepository: Send {
    fn display_fields(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<DisplayFields, CoreError>> + Send;

    fn replace_display_fields(
        &mut self,
        organization_id: OrganizationId,
        fields: &DisplayFields,
    ) -> impl Future<Output = Result<DisplayFields, CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait SupportApiKeyRepository: Send {
    fn create(
        &mut self,
        key: &SupportApiKey,
    ) -> impl Future<Output = Result<SupportApiKey, CoreError>> + Send;

    fn list(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<SupportApiKey>, CoreError>> + Send;

    fn find_by_prefix(
        &mut self,
        prefix: &str,
    ) -> impl Future<Output = Result<Option<SupportApiKey>, CoreError>> + Send;

    fn revoke(
        &mut self,
        organization_id: OrganizationId,
        id: SupportApiKeyId,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn touch_last_used(
        &mut self,
        id: SupportApiKeyId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
