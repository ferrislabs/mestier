use chrono::{NaiveDate, NaiveTime};
use chrono_tz::Tz;

use crate::{
    CustomerContextId, CustomerId, MemberId, OrganizationId, ProjectId,
    domain::task_recurrence::{RecurrenceRule, TaskRecurrenceId},
};

#[derive(Debug, Clone)]
pub struct CreateTaskRecurrenceCommand {
    pub organization_id: OrganizationId,
    pub rule: RecurrenceRule,
    pub starts_on: NaiveDate,
    pub ends_on: Option<NaiveDate>,
    pub timezone: Tz,
    pub start_time: NaiveTime,
    pub duration_minutes: i32,
    pub all_day: bool,
    pub title: String,
    pub description: Option<String>,
    pub blocks_availability: bool,
    pub customer_id: Option<CustomerId>,
    pub customer_context_id: Option<CustomerContextId>,
    pub project_id: Option<ProjectId>,
    pub assignee_member_ids: Vec<MemberId>,
}

/// Carries a `PATCH` on the recurrence itself: every field is optional and
/// only the ones present are applied. Nullable fields need the double
/// option, same convention as `task::commands::PatchTaskCommand`.
///
/// Patching the rule or the template only changes occurrences materialized
/// afterwards — see `TaskRecurrence`'s own doc.
#[derive(Debug, Clone)]
pub struct PatchTaskRecurrenceCommand {
    pub id: TaskRecurrenceId,
    pub rule: Option<RecurrenceRule>,
    pub ends_on: Option<Option<NaiveDate>>,
    pub start_time: Option<NaiveTime>,
    pub duration_minutes: Option<i32>,
    pub all_day: Option<bool>,
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub blocks_availability: Option<bool>,
    pub project_id: Option<Option<ProjectId>>,
    pub assignee_member_ids: Option<Vec<MemberId>>,
}

impl PatchTaskRecurrenceCommand {
    pub fn new(id: TaskRecurrenceId) -> Self {
        Self {
            id,
            rule: None,
            ends_on: None,
            start_time: None,
            duration_minutes: None,
            all_day: None,
            title: None,
            description: None,
            blocks_availability: None,
            project_id: None,
            assignee_member_ids: None,
        }
    }
}
