use chrono::{DateTime, Utc};

use crate::{
    CustomerContextId, CustomerId, OrganizationId, QuoteId,
    domain::work_order::{AssigneeRef, WorkOrderId, WorkOrderStatus},
};

#[derive(Debug, Clone)]
pub struct CreateWorkOrderCommand {
    pub organization_id: OrganizationId,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    pub quote_id: Option<QuoteId>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub all_day: bool,
    pub title: Option<String>,
    pub note: Option<String>,
}

/// Carries a `PATCH`: every field is optional and only the ones present are
/// applied. `title`/`note` are themselves nullable, so they need the double
/// option — `None` means "leave unchanged", `Some(None)` means "clear it",
/// `Some(Some(value))` means "set it".
#[derive(Debug, Clone)]
pub struct PatchWorkOrderCommand {
    pub id: WorkOrderId,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub all_day: Option<bool>,
    pub status: Option<WorkOrderStatus>,
    pub title: Option<Option<String>>,
    pub note: Option<Option<String>>,
    /// The complete replacement list of assignees, or `None` to leave the
    /// current assignments untouched. Never a delta.
    pub assignees: Option<Vec<AssigneeRef>>,
}

impl PatchWorkOrderCommand {
    /// A no-op patch targeting `id`: every field left unset. Tests and
    /// callers flip on only the fields they mean to change.
    pub fn new(id: WorkOrderId) -> Self {
        Self {
            id,
            starts_at: None,
            ends_at: None,
            all_day: None,
            status: None,
            title: None,
            note: None,
            assignees: None,
        }
    }
}
