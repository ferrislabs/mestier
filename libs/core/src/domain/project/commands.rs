use crate::{CustomerContextId, CustomerId, OrganizationId, ProjectId, QuoteId};

#[derive(Debug, Clone)]
pub struct CreateProjectCommand {
    pub organization_id: OrganizationId,
    pub name: String,
    pub customer_id: Option<CustomerId>,
    pub customer_context_id: Option<CustomerContextId>,
    pub quote_id: Option<QuoteId>,
}

/// Every field is the new complete value, never a delta: clearing the customer
/// means sending `None`, exactly like the `PATCH` contract on tasks treats
/// `assignees` as the full list.
#[derive(Debug, Clone)]
pub struct UpdateProjectCommand {
    pub id: ProjectId,
    pub name: String,
    pub customer_id: Option<CustomerId>,
    pub customer_context_id: Option<CustomerContextId>,
    pub quote_id: Option<QuoteId>,
}
