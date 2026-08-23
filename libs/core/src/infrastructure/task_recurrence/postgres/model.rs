use std::str::FromStr;

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    CustomerContextId, CustomerId, MemberId, OrganizationId, ProjectId, TaskRecurrence,
    TaskRecurrenceId,
    domain::task_recurrence::{RecurrenceFrequency, RecurrenceRule, weekday_from_iso},
};

#[derive(Debug, Clone)]
pub struct TaskRecurrenceRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub frequency: String,
    pub weekly_weekdays: Option<Vec<i16>>,
    pub monthly_day: Option<i16>,
    pub starts_on: NaiveDate,
    pub ends_on: Option<NaiveDate>,
    pub horizon_filled_to: NaiveDate,
    pub timezone: String,
    pub start_time: NaiveTime,
    pub duration_minutes: i32,
    pub all_day: bool,
    pub title: String,
    pub description: Option<String>,
    pub blocks_availability: bool,
    pub customer_id: Option<Uuid>,
    pub customer_context_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub assignee_member_ids: Vec<Uuid>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TaskRecurrenceRow {
    pub fn into_recurrence(self) -> Result<TaskRecurrence, CoreError> {
        let frequency = RecurrenceFrequency::from_str(&self.frequency).map_err(|e| {
            CoreError::Internal(format!("invalid recurrence frequency in database: {e}"))
        })?;

        let rule = match frequency {
            RecurrenceFrequency::Daily => RecurrenceRule::Daily,
            RecurrenceFrequency::Weekly => {
                let raw = self.weekly_weekdays.ok_or_else(|| {
                    CoreError::Internal(
                        "a WEEKLY recurrence in database with no weekly_weekdays".to_owned(),
                    )
                })?;
                let weekdays = raw
                    .into_iter()
                    .map(weekday_from_iso)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| {
                        CoreError::Internal(format!("invalid weekday in database: {e}"))
                    })?;
                RecurrenceRule::Weekly { weekdays }
            }
            RecurrenceFrequency::Monthly => {
                let day_of_month = self.monthly_day.ok_or_else(|| {
                    CoreError::Internal(
                        "a MONTHLY recurrence in database with no monthly_day".to_owned(),
                    )
                })?;
                RecurrenceRule::Monthly {
                    day_of_month: day_of_month as u8,
                }
            }
        };

        let timezone = self.timezone.parse().map_err(|_| {
            CoreError::Internal(format!(
                "invalid IANA timezone `{}` in database",
                self.timezone
            ))
        })?;

        Ok(TaskRecurrence {
            id: TaskRecurrenceId(self.id),
            organization_id: OrganizationId(self.org_id),
            rule,
            starts_on: self.starts_on,
            ends_on: self.ends_on,
            horizon_filled_to: self.horizon_filled_to,
            timezone,
            start_time: self.start_time,
            duration_minutes: self.duration_minutes,
            all_day: self.all_day,
            title: self.title,
            description: self.description,
            blocks_availability: self.blocks_availability,
            customer_id: self.customer_id.map(CustomerId),
            customer_context_id: self.customer_context_id.map(CustomerContextId),
            project_id: self.project_id.map(ProjectId),
            assignee_member_ids: self.assignee_member_ids.into_iter().map(MemberId).collect(),
            deleted_at: self.deleted_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
