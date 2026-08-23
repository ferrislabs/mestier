use chrono::{NaiveDate, Utc};
use common::CoreError;
use mestier_macros::transactional;
use serde_json::json;
use uuid::Uuid;

use crate::{
    CreateWorkflowCommand, Edge, Graph, OrganizationId, PlacedConnector, RunStatus,
    SaveWorkflowVersionCommand, TaskRecurrence, TaskRecurrenceId,
    application::MestierUseCase,
    domain::task_recurrence::{
        commands::{CreateTaskRecurrenceCommand, PatchTaskRecurrenceCommand},
        service::{DEFAULT_HORIZON_DAYS, TaskRecurrenceService},
    },
};

#[cfg(test)]
mod tests;

/// The name of the system-managed workflow every organization gets lazily,
/// the first time it has a recurrence whose horizon needs extending. Not
/// meant to be edited or run by a human — `ensure_recurrence_horizon_runs`
/// is its only caller, and it exists at all so the horizon-extension pass
/// can reuse the run engine's already-tested claiming, backoff and
/// stale-claim recovery (`automation.run`) instead of standing up a second
/// scheduler next to it.
const RECURRENCE_HORIZON_WORKFLOW_NAME: &str = "System: recurrence horizon";

impl MestierUseCase {
    /// Creates a recurrence and materializes its occurrences up to the
    /// default horizon — see `TaskRecurrenceService::create_recurrence`.
    /// Every occurrence is materialized inside this one transaction, so a
    /// validation failure partway through (an unknown assignee, say) leaves
    /// nothing behind.
    #[transactional(task_recurrence, task, member)]
    pub async fn create_task_recurrence(
        &self,
        command: CreateTaskRecurrenceCommand,
    ) -> Result<TaskRecurrence, CoreError> {
        let mut service = TaskRecurrenceService::new(
            task_recurrence_repository,
            task_repository,
            member_repository,
        );
        service
            .create_recurrence(command, DEFAULT_HORIZON_DAYS)
            .await
    }

    #[transactional(task_recurrence, task, member)]
    pub async fn get_task_recurrence(
        &self,
        id: TaskRecurrenceId,
    ) -> Result<TaskRecurrence, CoreError> {
        let mut service = TaskRecurrenceService::new(
            task_recurrence_repository,
            task_repository,
            member_repository,
        );
        service.get_recurrence(id).await
    }

    #[transactional(task_recurrence, task, member)]
    pub async fn list_task_recurrences(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<TaskRecurrence>, CoreError> {
        let mut service = TaskRecurrenceService::new(
            task_recurrence_repository,
            task_repository,
            member_repository,
        );
        service.list_recurrences(organization_id).await
    }

    #[transactional(task_recurrence, task, member)]
    pub async fn patch_task_recurrence(
        &self,
        command: PatchTaskRecurrenceCommand,
    ) -> Result<TaskRecurrence, CoreError> {
        let mut service = TaskRecurrenceService::new(
            task_recurrence_repository,
            task_repository,
            member_repository,
        );
        service.patch_recurrence(command).await
    }

    /// Soft-deletes the recurrence and its future occurrences, in one
    /// transaction — see `TaskRecurrenceService::delete_recurrence`.
    #[transactional(task_recurrence, task, member)]
    pub async fn delete_task_recurrence(&self, id: TaskRecurrenceId) -> Result<(), CoreError> {
        let mut service = TaskRecurrenceService::new(
            task_recurrence_repository,
            task_repository,
            member_repository,
        );
        service.delete_recurrence(id).await
    }

    /// Extends every one of `organization_id`'s recurrences whose horizon
    /// needs pushing forward, in one transaction — see
    /// `TaskRecurrenceService::extend_organization_horizons`. The only
    /// caller is `TaskRecurrenceExtendHorizonConnector`, itself only ever
    /// invoked by the run engine against a run this module created (see
    /// [`Self::ensure_recurrence_horizon_runs`]).
    #[transactional(task_recurrence, task, member)]
    pub async fn extend_recurrence_horizons_for_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<u64, CoreError> {
        let mut service = TaskRecurrenceService::new(
            task_recurrence_repository,
            task_repository,
            member_repository,
        );
        service.extend_organization_horizons(organization_id).await
    }

    #[transactional(task_recurrence, task, member)]
    async fn organizations_needing_recurrence_horizon_extension(
        &self,
        today: NaiveDate,
    ) -> Result<Vec<OrganizationId>, CoreError> {
        let mut service = TaskRecurrenceService::new(
            task_recurrence_repository,
            task_repository,
            member_repository,
        );
        service.organizations_needing_extension(today).await
    }

    /// Finds this organization's system recurrence-horizon workflow,
    /// creating it (and its one-connector version) on first use. Looked up
    /// by name rather than a stored id: workflows are already listed in
    /// full per organization, there are only ever a handful per
    /// organization, and a dedicated column would be one more place for the
    /// two to drift apart.
    async fn find_or_create_recurrence_horizon_workflow(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Uuid, CoreError> {
        let workflows = self.list_workflows(organization_id).await?;
        if let Some(existing) = workflows
            .iter()
            .find(|workflow| workflow.name == RECURRENCE_HORIZON_WORKFLOW_NAME)
        {
            return Ok(existing.id);
        }

        let workflow = self
            .create_workflow(CreateWorkflowCommand {
                org_id: organization_id,
                name: RECURRENCE_HORIZON_WORKFLOW_NAME.to_owned(),
                description: Some(
                    "Extends every recurring task's materialized horizon. \
                     System-managed: not meant to be edited."
                        .to_owned(),
                ),
            })
            .await?;

        self.save_workflow_version(SaveWorkflowVersionCommand {
            org_id: organization_id,
            workflow_id: workflow.id,
            graph: Graph {
                connectors: vec![PlacedConnector {
                    id: "extend_horizon".to_owned(),
                    kind: "mestier.task_recurrence.extend_horizon".to_owned(),
                    version: 1,
                    credential_id: None,
                    config: serde_json::Map::new(),
                }],
                edges: Vec::<Edge>::new(),
            },
            created_by: None,
        })
        .await?;

        Ok(workflow.id)
    }

    /// Whether `workflow_id` already has a run that has not settled —
    /// `pending` (claimable, or waiting on backoff) or `running` (claimed,
    /// mid-slice). Checked before creating a new one so a quiet organization
    /// stays quiet: the worker ticks every few seconds, far more often than
    /// a horizon actually needs pushing, and without this check every tick
    /// between two extension passes would queue another run for the same
    /// work.
    async fn has_active_recurrence_horizon_run(
        &self,
        organization_id: OrganizationId,
        workflow_id: Uuid,
    ) -> Result<bool, CoreError> {
        let runs = self.list_runs(organization_id).await?;
        Ok(runs.iter().any(|run| {
            run.workflow_id == workflow_id
                && matches!(run.status, RunStatus::Pending | RunStatus::Running)
        }))
    }

    /// Ensures every organization whose recurrence horizon needs pushing
    /// forward soon has exactly one run queued to do it — the periodic
    /// counterpart to [`Self::dispatch_pending_events`] in the automation
    /// worker's own tick (see `infrastructure::automation::worker::run_automation_worker`),
    /// reusing the same durable `automation.run` queue rather than standing
    /// up a second scheduler.
    ///
    /// An organization with no recurrences never reaches this loop at all —
    /// `organizations_needing_recurrence_horizon_extension` only ever names
    /// organizations that have at least one — so it can never produce a run.
    ///
    /// Returns how many runs were newly created.
    pub async fn ensure_recurrence_horizon_runs(&self) -> Result<usize, CoreError> {
        let today = Utc::now().date_naive();
        let organizations = self
            .organizations_needing_recurrence_horizon_extension(today)
            .await?;

        let mut created = 0;
        for organization_id in organizations {
            let workflow_id = self
                .find_or_create_recurrence_horizon_workflow(organization_id)
                .await?;

            if self
                .has_active_recurrence_horizon_run(organization_id, workflow_id)
                .await?
            {
                continue;
            }

            self.start_run(organization_id, workflow_id, json!({}))
                .await?;
            created += 1;
        }

        Ok(created)
    }
}
