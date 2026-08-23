use chrono::NaiveDate;
use common::CoreError;

use crate::{OrganizationId, TaskRecurrence, TaskRecurrenceId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait TaskRecurrenceRepository: Send {
    fn insert(
        &mut self,
        recurrence: &TaskRecurrence,
    ) -> impl Future<Output = Result<TaskRecurrence, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: TaskRecurrenceId,
    ) -> impl Future<Output = Result<Option<TaskRecurrence>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<TaskRecurrence>, CoreError>> + Send;

    /// Persists the rule/template fields a `PATCH` may change. Never moves
    /// `horizon_filled_to` — that is [`Self::advance_horizon`]'s job alone,
    /// so a template edit can never accidentally rewind or fast-forward the
    /// watermark.
    fn update(
        &mut self,
        recurrence: &TaskRecurrence,
    ) -> impl Future<Output = Result<TaskRecurrence, CoreError>> + Send;

    /// Moves the watermark forward. Called in the same transaction as the
    /// occurrence rows it accounts for, so a failure partway through a fill
    /// leaves the watermark exactly where it was — the next pass redoes the
    /// missing part rather than skipping it.
    fn advance_horizon(
        &mut self,
        id: TaskRecurrenceId,
        horizon_filled_to: NaiveDate,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Soft-deletes the recurrence itself. Future occurrence tasks are
    /// deleted by the caller alongside this (see
    /// `service::TaskRecurrenceService::delete_recurrence`); past ones keep
    /// their `recurrence_id`, which stays valid because this is a soft
    /// delete, not a physical one — the composite FK on `tasks.recurrence_id`
    /// would otherwise be left dangling.
    fn soft_delete(
        &mut self,
        id: TaskRecurrenceId,
        deleted_at: chrono::DateTime<chrono::Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
