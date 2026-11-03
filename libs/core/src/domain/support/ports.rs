use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{MemberId, OrganizationId, Ticket, TicketId, TicketNumber, TicketStatus};

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
