//! The visit loop: the field proposes a date on its own task, the manager
//! validates or rejects it, or enters one directly. Only a validated visit
//! moves a line's due date (invariant 2); each resolution persists its event
//! in the transaction that makes it (invariant 8).

use chrono::{NaiveDate, Utc};
use common::{CoreError, generate_uuid_v7};
use events::EventEmitter;

use crate::{
    Intervention, InterventionId, InterventionStatus, MaintenanceContract, MaintenanceLineId,
    MemberId, OrganizationId, TaskId, check_intervention_date,
    domain::maintenance::{
        events::{InterventionProposed, InterventionRejected, InterventionValidated},
        ports::{InterventionRepository, MaintenanceContractRepository},
    },
};

pub struct InterventionService<C, I, E>
where
    C: MaintenanceContractRepository,
    I: InterventionRepository,
    E: EventEmitter,
{
    contracts: C,
    interventions: I,
    emitter: E,
}

impl<C, I, E> InterventionService<C, I, E>
where
    C: MaintenanceContractRepository,
    I: InterventionRepository,
    E: EventEmitter,
{
    pub fn new(contracts: C, interventions: I, emitter: E) -> Self {
        Self {
            contracts,
            interventions,
            emitter,
        }
    }

    /// The contract carrying `line_id`, or `NotFound` when the line does not
    /// exist or belongs to another organization.
    pub async fn contract_of_line(
        &mut self,
        organization_id: OrganizationId,
        line_id: MaintenanceLineId,
    ) -> Result<MaintenanceContract, CoreError> {
        self.contracts
            .find_by_line(line_id)
            .await?
            .filter(|contract| contract.organization_id == organization_id)
            .ok_or(CoreError::NotFound)
    }

    pub async fn get(
        &mut self,
        organization_id: OrganizationId,
        id: InterventionId,
    ) -> Result<Intervention, CoreError> {
        self.interventions
            .find_by_id(id)
            .await?
            .filter(|intervention| intervention.organization_id == organization_id)
            .ok_or(CoreError::NotFound)
    }

    pub async fn list_by_status(
        &mut self,
        organization_id: OrganizationId,
        status: InterventionStatus,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Intervention>, u64), CoreError> {
        self.interventions
            .list_by_status(organization_id, status, limit, offset)
            .await
    }

    /// A manager's direct entry: validated at once, so it emits `validated`
    /// and never `proposed`.
    pub async fn record(
        &mut self,
        contract: &MaintenanceContract,
        line_id: MaintenanceLineId,
        by: MemberId,
        done_on: NaiveDate,
        today: NaiveDate,
    ) -> Result<Intervention, CoreError> {
        check_intervention_date(contract.season, done_on, today)?;

        let now = Utc::now();
        let intervention = self
            .interventions
            .insert(&Intervention {
                id: InterventionId(generate_uuid_v7()),
                organization_id: contract.organization_id,
                line_id,
                done_on,
                status: InterventionStatus::Validated,
                task_id: None,
                proposed_by: by,
                resolved_by: Some(by),
                resolved_at: Some(now),
                note: None,
                created_at: now,
            })
            .await?;

        self.emitter.emit(
            intervention.organization_id,
            &InterventionValidated {
                intervention: intervention.clone(),
            },
        )?;
        Ok(intervention)
    }

    /// A field worker's date on a maintenance task assigned to them.
    ///
    /// Every refusal is `NotFound`: a task that is missing, from another
    /// organization, not a maintenance task, or assigned to somebody else is
    /// the same thing to this caller, and saying which would leak it.
    pub async fn propose(
        &mut self,
        organization_id: OrganizationId,
        task_id: TaskId,
        by: MemberId,
        done_on: NaiveDate,
        today: NaiveDate,
    ) -> Result<Intervention, CoreError> {
        let context = self
            .interventions
            .find_task_context(task_id)
            .await?
            .filter(|context| context.organization_id == organization_id)
            .filter(|context| context.assignee_member_ids.contains(&by))
            .ok_or(CoreError::NotFound)?;
        let line_id = context.line_id.ok_or(CoreError::NotFound)?;

        let contract = self.contract_of_line(organization_id, line_id).await?;
        check_intervention_date(contract.season, done_on, today)?;

        let intervention = self
            .interventions
            .insert(&Intervention {
                id: InterventionId(generate_uuid_v7()),
                organization_id,
                line_id,
                done_on,
                status: InterventionStatus::Proposed,
                task_id: Some(task_id),
                proposed_by: by,
                resolved_by: None,
                resolved_at: None,
                note: None,
                created_at: Utc::now(),
            })
            .await?;

        self.emitter.emit(
            organization_id,
            &InterventionProposed {
                intervention: intervention.clone(),
            },
        )?;
        Ok(intervention)
    }

    pub async fn validate(
        &mut self,
        mut intervention: Intervention,
        by: MemberId,
    ) -> Result<Intervention, CoreError> {
        resolve(&mut intervention, InterventionStatus::Validated, by, None)?;
        let saved = self.interventions.update_resolution(&intervention).await?;

        self.emitter.emit(
            saved.organization_id,
            &InterventionValidated {
                intervention: saved.clone(),
            },
        )?;
        Ok(saved)
    }

    pub async fn reject(
        &mut self,
        mut intervention: Intervention,
        by: MemberId,
        note: Option<String>,
    ) -> Result<Intervention, CoreError> {
        if note.as_deref().is_some_and(|text| text.trim().is_empty()) {
            return Err(CoreError::Conflict(
                "a rejection note cannot be blank".to_owned(),
            ));
        }
        resolve(&mut intervention, InterventionStatus::Rejected, by, note)?;
        let saved = self.interventions.update_resolution(&intervention).await?;

        self.emitter.emit(
            saved.organization_id,
            &InterventionRejected {
                intervention: saved.clone(),
            },
        )?;
        Ok(saved)
    }
}

/// A resolved visit stays resolved: validating twice, or rejecting a
/// validated one, is a 409 rather than a silent rewrite of history.
fn resolve(
    intervention: &mut Intervention,
    status: InterventionStatus,
    by: MemberId,
    note: Option<String>,
) -> Result<(), CoreError> {
    if intervention.status != InterventionStatus::Proposed {
        return Err(CoreError::Conflict(
            "this intervention has already been resolved".to_owned(),
        ));
    }
    intervention.status = status;
    intervention.resolved_by = Some(by);
    intervention.resolved_at = Some(Utc::now());
    intervention.note = note;
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use events::testing::RecordingEmitter;
    use uuid::Uuid;

    use super::*;
    use crate::{
        Cadence, ContractStatus, CustomerContextId, CustomerId, IntervalDays, IsoWeekNumber,
        MaintenanceContractId, MaintenanceLine, ProjectId, Season,
        domain::maintenance::ports::{
            MockInterventionRepository, MockMaintenanceContractRepository, TaskMaintenanceContext,
        },
    };

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn member() -> MemberId {
        MemberId(Uuid::new_v4())
    }

    fn contract(
        organization_id: OrganizationId,
        line_id: MaintenanceLineId,
    ) -> MaintenanceContract {
        let now: DateTime<Utc> = Utc::now();
        MaintenanceContract {
            id: MaintenanceContractId(Uuid::new_v4()),
            organization_id,
            customer_id: CustomerId(Uuid::new_v4()),
            customer_context_id: CustomerContextId(Uuid::new_v4()),
            project_id: ProjectId(Uuid::new_v4()),
            reference: None,
            season: Season::new(
                2026,
                IsoWeekNumber::try_from(12).unwrap(),
                IsoWeekNumber::try_from(48).unwrap(),
            )
            .unwrap(),
            manager_member_id: None,
            amount_cents: None,
            quote_id: None,
            notes: None,
            status: ContractStatus::Active,
            lines: vec![MaintenanceLine {
                id: line_id,
                label: "Tonte".to_owned(),
                cadence: Cadence::Rolling {
                    every_days: IntervalDays::try_from(14).unwrap(),
                },
                minutes_per_intervention: None,
            }],
            archived_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn proposed(organization_id: OrganizationId, line_id: MaintenanceLineId) -> Intervention {
        Intervention {
            id: InterventionId(Uuid::new_v4()),
            organization_id,
            line_id,
            done_on: date(2026, 10, 14),
            status: InterventionStatus::Proposed,
            task_id: Some(TaskId(Uuid::new_v4())),
            proposed_by: member(),
            resolved_by: None,
            resolved_at: None,
            note: None,
            created_at: Utc::now(),
        }
    }

    fn echo_insert(interventions: &mut MockInterventionRepository) {
        interventions.expect_insert().returning(|intervention| {
            let intervention = intervention.clone();
            Box::pin(async move { Ok(intervention) })
        });
    }

    fn echo_update(interventions: &mut MockInterventionRepository) {
        interventions
            .expect_update_resolution()
            .returning(|intervention| {
                let intervention = intervention.clone();
                Box::pin(async move { Ok(intervention) })
            });
    }

    #[tokio::test]
    async fn spec_ent_7_the_assignee_proposes_and_the_event_is_published() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let line_id = MaintenanceLineId(Uuid::new_v4());
        let task_id = TaskId(Uuid::new_v4());
        let bernard = member();
        let mut interventions = MockInterventionRepository::new();
        interventions
            .expect_find_task_context()
            .returning(move |_| {
                Box::pin(async move {
                    Ok(Some(TaskMaintenanceContext {
                        organization_id,
                        line_id: Some(line_id),
                        assignee_member_ids: vec![bernard],
                    }))
                })
            });
        echo_insert(&mut interventions);
        let mut contracts = MockMaintenanceContractRepository::new();
        contracts.expect_find_by_line().returning(move |_| {
            Box::pin(async move { Ok(Some(contract(organization_id, line_id))) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = InterventionService::new(contracts, interventions, &emitter);

        let proposal = service
            .propose(
                organization_id,
                task_id,
                bernard,
                date(2026, 10, 14),
                date(2026, 10, 14),
            )
            .await
            .unwrap();

        assert_eq!(proposal.status, InterventionStatus::Proposed);
        assert_eq!(proposal.task_id, Some(task_id));
        assert_eq!(proposal.resolved_by, None);
        assert_eq!(emitter.names(), ["maintenance_intervention.proposed"]);
    }

    #[tokio::test]
    async fn spec_ent_8_somebody_else_assigned_gets_not_found_and_nothing_is_written() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let paul = member();
        let bernard = member();
        let mut interventions = MockInterventionRepository::new();
        interventions
            .expect_find_task_context()
            .returning(move |_| {
                Box::pin(async move {
                    Ok(Some(TaskMaintenanceContext {
                        organization_id,
                        line_id: Some(MaintenanceLineId(Uuid::new_v4())),
                        assignee_member_ids: vec![paul],
                    }))
                })
            });
        interventions.expect_insert().never();
        let emitter = RecordingEmitter::new();
        let mut service = InterventionService::new(
            MockMaintenanceContractRepository::new(),
            interventions,
            &emitter,
        );

        let result = service
            .propose(
                organization_id,
                TaskId(Uuid::new_v4()),
                bernard,
                date(2026, 10, 14),
                date(2026, 10, 14),
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn a_task_that_is_not_a_maintenance_task_is_not_found() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let bernard = member();
        let mut interventions = MockInterventionRepository::new();
        interventions
            .expect_find_task_context()
            .returning(move |_| {
                Box::pin(async move {
                    Ok(Some(TaskMaintenanceContext {
                        organization_id,
                        line_id: None,
                        assignee_member_ids: vec![bernard],
                    }))
                })
            });
        let mut service = InterventionService::new(
            MockMaintenanceContractRepository::new(),
            interventions,
            RecordingEmitter::new(),
        );

        let result = service
            .propose(
                organization_id,
                TaskId(Uuid::new_v4()),
                bernard,
                date(2026, 10, 14),
                date(2026, 10, 14),
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn a_task_of_another_organization_is_not_found() {
        let bernard = member();
        let mut interventions = MockInterventionRepository::new();
        interventions
            .expect_find_task_context()
            .returning(move |_| {
                Box::pin(async move {
                    Ok(Some(TaskMaintenanceContext {
                        organization_id: OrganizationId(Uuid::new_v4()),
                        line_id: Some(MaintenanceLineId(Uuid::new_v4())),
                        assignee_member_ids: vec![bernard],
                    }))
                })
            });
        let mut service = InterventionService::new(
            MockMaintenanceContractRepository::new(),
            interventions,
            RecordingEmitter::new(),
        );

        let result = service
            .propose(
                OrganizationId(Uuid::new_v4()),
                TaskId(Uuid::new_v4()),
                bernard,
                date(2026, 10, 14),
                date(2026, 10, 14),
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn a_proposal_dated_in_the_future_is_a_conflict() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let line_id = MaintenanceLineId(Uuid::new_v4());
        let bernard = member();
        let mut interventions = MockInterventionRepository::new();
        interventions
            .expect_find_task_context()
            .returning(move |_| {
                Box::pin(async move {
                    Ok(Some(TaskMaintenanceContext {
                        organization_id,
                        line_id: Some(line_id),
                        assignee_member_ids: vec![bernard],
                    }))
                })
            });
        interventions.expect_insert().never();
        let mut contracts = MockMaintenanceContractRepository::new();
        contracts.expect_find_by_line().returning(move |_| {
            Box::pin(async move { Ok(Some(contract(organization_id, line_id))) })
        });
        let mut service =
            InterventionService::new(contracts, interventions, RecordingEmitter::new());

        let result = service
            .propose(
                organization_id,
                TaskId(Uuid::new_v4()),
                bernard,
                date(2026, 10, 20),
                date(2026, 10, 14),
            )
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn spec_ent_7_validating_resolves_the_proposal_and_publishes_validated() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let manager = member();
        let mut interventions = MockInterventionRepository::new();
        echo_update(&mut interventions);
        let emitter = RecordingEmitter::new();
        let mut service = InterventionService::new(
            MockMaintenanceContractRepository::new(),
            interventions,
            &emitter,
        );

        let saved = service
            .validate(
                proposed(organization_id, MaintenanceLineId(Uuid::new_v4())),
                manager,
            )
            .await
            .unwrap();

        assert_eq!(saved.status, InterventionStatus::Validated);
        assert_eq!(saved.resolved_by, Some(manager));
        assert!(saved.resolved_at.is_some());
        assert_eq!(emitter.names(), ["maintenance_intervention.validated"]);
    }

    #[tokio::test]
    async fn rejecting_keeps_the_note_and_publishes_rejected() {
        let mut interventions = MockInterventionRepository::new();
        echo_update(&mut interventions);
        let emitter = RecordingEmitter::new();
        let mut service = InterventionService::new(
            MockMaintenanceContractRepository::new(),
            interventions,
            &emitter,
        );

        let saved = service
            .reject(
                proposed(
                    OrganizationId(Uuid::new_v4()),
                    MaintenanceLineId(Uuid::new_v4()),
                ),
                member(),
                Some("Mauvaise date".to_owned()),
            )
            .await
            .unwrap();

        assert_eq!(saved.status, InterventionStatus::Rejected);
        assert_eq!(saved.note.as_deref(), Some("Mauvaise date"));
        assert_eq!(emitter.names(), ["maintenance_intervention.rejected"]);
    }

    #[tokio::test]
    async fn a_blank_rejection_note_is_a_conflict() {
        let mut interventions = MockInterventionRepository::new();
        interventions.expect_update_resolution().never();
        let mut service = InterventionService::new(
            MockMaintenanceContractRepository::new(),
            interventions,
            RecordingEmitter::new(),
        );

        let result = service
            .reject(
                proposed(
                    OrganizationId(Uuid::new_v4()),
                    MaintenanceLineId(Uuid::new_v4()),
                ),
                member(),
                Some("   ".to_owned()),
            )
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn resolving_an_already_resolved_intervention_is_a_conflict() {
        for status in [InterventionStatus::Validated, InterventionStatus::Rejected] {
            let mut interventions = MockInterventionRepository::new();
            interventions.expect_update_resolution().never();
            let emitter = RecordingEmitter::new();
            let mut service = InterventionService::new(
                MockMaintenanceContractRepository::new(),
                interventions,
                &emitter,
            );
            let mut resolved = proposed(
                OrganizationId(Uuid::new_v4()),
                MaintenanceLineId(Uuid::new_v4()),
            );
            resolved.status = status;

            let validated = service.validate(resolved.clone(), member()).await;
            let rejected = service.reject(resolved, member(), None).await;

            assert!(
                matches!(validated, Err(CoreError::Conflict(_))),
                "{status:?}"
            );
            assert!(
                matches!(rejected, Err(CoreError::Conflict(_))),
                "{status:?}"
            );
            assert!(emitter.names().is_empty());
        }
    }

    #[tokio::test]
    async fn a_manager_entry_is_validated_at_once_and_publishes_only_validated() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let line_id = MaintenanceLineId(Uuid::new_v4());
        let manager = member();
        let mut interventions = MockInterventionRepository::new();
        echo_insert(&mut interventions);
        let emitter = RecordingEmitter::new();
        let mut service = InterventionService::new(
            MockMaintenanceContractRepository::new(),
            interventions,
            &emitter,
        );

        let saved = service
            .record(
                &contract(organization_id, line_id),
                line_id,
                manager,
                date(2026, 10, 10),
                date(2026, 10, 12),
            )
            .await
            .unwrap();

        assert_eq!(saved.status, InterventionStatus::Validated);
        assert_eq!(saved.task_id, None);
        assert_eq!(saved.resolved_by, Some(manager));
        assert!(saved.resolved_at.is_some());
        assert_eq!(emitter.names(), ["maintenance_intervention.validated"]);
    }

    #[tokio::test]
    async fn a_manager_entry_outside_the_season_is_a_conflict() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let line_id = MaintenanceLineId(Uuid::new_v4());
        let mut interventions = MockInterventionRepository::new();
        interventions.expect_insert().never();
        let mut service = InterventionService::new(
            MockMaintenanceContractRepository::new(),
            interventions,
            RecordingEmitter::new(),
        );

        let result = service
            .record(
                &contract(organization_id, line_id),
                line_id,
                member(),
                date(2026, 2, 1),
                date(2026, 10, 12),
            )
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn an_intervention_of_another_organization_is_not_found() {
        let mut interventions = MockInterventionRepository::new();
        interventions.expect_find_by_id().returning(|_| {
            let other = proposed(
                OrganizationId(Uuid::new_v4()),
                MaintenanceLineId(Uuid::new_v4()),
            );
            Box::pin(async move { Ok(Some(other)) })
        });
        let mut service = InterventionService::new(
            MockMaintenanceContractRepository::new(),
            interventions,
            RecordingEmitter::new(),
        );

        let result = service
            .get(
                OrganizationId(Uuid::new_v4()),
                InterventionId(Uuid::new_v4()),
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn a_line_of_another_organization_is_not_found() {
        let line_id = MaintenanceLineId(Uuid::new_v4());
        let mut contracts = MockMaintenanceContractRepository::new();
        contracts.expect_find_by_line().returning(move |_| {
            let other = contract(OrganizationId(Uuid::new_v4()), line_id);
            Box::pin(async move { Ok(Some(other)) })
        });
        let mut service = InterventionService::new(
            contracts,
            MockInterventionRepository::new(),
            RecordingEmitter::new(),
        );

        let result = service
            .contract_of_line(OrganizationId(Uuid::new_v4()), line_id)
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }
}
