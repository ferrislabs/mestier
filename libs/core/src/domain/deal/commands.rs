use authz::Subject;
use chrono::NaiveDate;

use crate::{CustomerId, DealId, DealStageName, LeadSource, MemberId, OrganizationId};

#[derive(Debug, Clone)]
pub struct CreateDealCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub customer_id: CustomerId,
    pub title: String,
    pub amount_cents: i64,
    pub expected_close_date: Option<NaiveDate>,
    pub owner_member_id: Option<MemberId>,
    pub source: Option<LeadSource>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateDealCommand {
    pub actor: Subject,
    pub id: DealId,
    pub customer_id: CustomerId,
    pub title: String,
    pub amount_cents: i64,
    pub expected_close_date: Option<NaiveDate>,
    pub owner_member_id: Option<MemberId>,
    pub source: Option<LeadSource>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MoveDealCommand {
    pub actor: Subject,
    pub id: DealId,
    pub stage: DealStageName,
    pub lost_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DealFilter {
    pub organization_id: OrganizationId,
    pub customer_id: Option<CustomerId>,
    pub stage: Option<DealStageName>,
    pub query: Option<String>,
}
