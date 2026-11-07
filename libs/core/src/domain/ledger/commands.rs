use authz::Subject;
use chrono::NaiveDate;
use common::UserId;

use crate::{CustomerId, InvoiceId, OrganizationId, PaymentMethod, ReceiptId};

#[derive(Debug, Clone)]
pub struct RecordReceiptCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub customer_id: Option<CustomerId>,
    pub amount_cents: i32,
    pub paid_on: NaiveDate,
    pub method: PaymentMethod,
    pub reference: Option<String>,
    pub note: Option<String>,
    pub recorded_by: UserId,
}

#[derive(Debug, Clone)]
pub struct AllocateReceiptCommand {
    pub actor: Subject,
    pub id: ReceiptId,
    pub invoice_id: InvoiceId,
    pub recorded_by: UserId,
}
