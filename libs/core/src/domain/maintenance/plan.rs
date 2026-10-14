//! Turning recap lines into backlog tasks: one undated task per line, in the
//! contract's project so its cost lands on the contract (ADR 0002), carrying
//! the site and the line so the recap sees the line as planned.

use std::collections::HashSet;

use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    IsoWeekNumber, MaintenanceLine, MaintenanceLineId, OrganizationId, Task, TaskId, TaskStatus,
    domain::{
        maintenance::{ContractStatus, MaintenanceContract, ports::MaintenanceContractRepository},
        task::ports::TaskRepository,
    },
};

pub struct PlanService<C, T>
where
    C: MaintenanceContractRepository,
    T: TaskRepository,
{
    contracts: C,
    tasks: T,
}

impl<C, T> PlanService<C, T>
where
    C: MaintenanceContractRepository,
    T: TaskRepository,
{
    pub fn new(contracts: C, tasks: T) -> Self {
        Self { contracts, tasks }
    }

    /// One backlog task per distinct line, in the order given.
    ///
    /// All or nothing: a line that is unknown or belongs to another
    /// organization is `NotFound`, a line of an archived or draft contract
    /// or one an open task already covers is a `Conflict`, and in either
    /// case no task is created.
    pub async fn plan(
        &mut self,
        organization_id: OrganizationId,
        line_ids: &[MaintenanceLineId],
        year: i32,
        week: IsoWeekNumber,
    ) -> Result<Vec<Task>, CoreError> {
        let mut seen = HashSet::with_capacity(line_ids.len());
        let wanted: Vec<MaintenanceLineId> = line_ids
            .iter()
            .copied()
            .filter(|line_id| seen.insert(*line_id))
            .collect();

        let mut targets = Vec::with_capacity(wanted.len());
        for line_id in &wanted {
            let contract = self
                .contracts
                .find_by_line(*line_id)
                .await?
                .filter(|contract| contract.organization_id == organization_id)
                .ok_or(CoreError::NotFound)?;
            ensure_plannable(&contract)?;
            let line = contract
                .lines
                .iter()
                .find(|line| line.id == *line_id)
                .cloned()
                .ok_or(CoreError::NotFound)?;
            targets.push((contract, line));
        }

        let open = self
            .contracts
            .lines_with_open_task(organization_id, &wanted)
            .await?;
        if !open.is_empty() {
            return Err(CoreError::Conflict(
                "a line is already planned: an open task carries it".to_owned(),
            ));
        }

        let mut created = Vec::with_capacity(targets.len());
        for (contract, line) in targets {
            created.push(
                self.tasks
                    .insert(&backlog_task(&contract, &line, year, week))
                    .await?,
            );
        }
        Ok(created)
    }
}

fn ensure_plannable(contract: &MaintenanceContract) -> Result<(), CoreError> {
    if contract.archived_at.is_some() {
        return Err(CoreError::Conflict(
            "the contract is archived: its lines cannot be planned".to_owned(),
        ));
    }
    if contract.status == ContractStatus::Draft {
        return Err(CoreError::Conflict(
            "the contract is still a draft: its lines cannot be planned".to_owned(),
        ));
    }
    Ok(())
}

fn backlog_task(
    contract: &MaintenanceContract,
    line: &MaintenanceLine,
    year: i32,
    week: IsoWeekNumber,
) -> Task {
    let now = Utc::now();
    Task {
        id: TaskId(generate_uuid_v7()),
        organization_id: contract.organization_id,
        parent_task_id: None,
        title: line.label.clone(),
        description: Some(format!("Entretien à prévoir en S{} {year}", week.get())),
        starts_at: None,
        ends_at: None,
        all_day: false,
        status: TaskStatus::Backlog,
        blocks_availability: false,
        customer_id: Some(contract.customer_id),
        customer_context_id: Some(contract.customer_context_id),
        quote_id: None,
        project_id: Some(contract.project_id),
        expenses_cents: 0,
        expenses_label: None,
        assignments: Vec::new(),
        board_rank: None,
        recurrence_id: None,
        occurrence_date: None,
        maintenance_line_id: Some(line.id),
        deleted_at: None,
        created_at: now,
        updated_at: now,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use uuid::Uuid;

    use super::*;
    use crate::{
        Cadence, CustomerContextId, CustomerId, MaintenanceContractId, ProjectId, Season,
        domain::{
            maintenance::{IntervalDays, ports::MockMaintenanceContractRepository},
            task::ports::MockTaskRepository,
        },
    };

    fn week(n: u8) -> IsoWeekNumber {
        IsoWeekNumber::try_from(n).unwrap()
    }

    fn contract(
        organization_id: OrganizationId,
        lines: &[MaintenanceLineId],
    ) -> MaintenanceContract {
        let now = Utc::now();
        MaintenanceContract {
            id: MaintenanceContractId(Uuid::new_v4()),
            organization_id,
            customer_id: CustomerId(Uuid::new_v4()),
            customer_context_id: CustomerContextId(Uuid::new_v4()),
            project_id: ProjectId(Uuid::new_v4()),
            reference: None,
            season: Season::new(2026, week(12), week(48)).unwrap(),
            manager_member_id: None,
            amount_cents: None,
            quote_id: None,
            notes: None,
            status: ContractStatus::Active,
            lines: lines
                .iter()
                .map(|id| MaintenanceLine {
                    id: *id,
                    label: format!("Prestation {}", id.0.simple()),
                    cadence: Cadence::Rolling {
                        every_days: IntervalDays::try_from(14).unwrap(),
                    },
                    minutes_per_intervention: Some(90),
                })
                .collect(),
            archived_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn contracts_with(
        organization_id: OrganizationId,
        lines: Vec<MaintenanceLineId>,
        open: HashSet<MaintenanceLineId>,
    ) -> (MockMaintenanceContractRepository, MaintenanceContract) {
        let contract = contract(organization_id, &lines);
        let found = contract.clone();
        let mut contracts = MockMaintenanceContractRepository::new();
        contracts.expect_find_by_line().returning(move |_| {
            let found = found.clone();
            Box::pin(async move { Ok(Some(found)) })
        });
        contracts
            .expect_lines_with_open_task()
            .returning(move |_, _| {
                let open = open.clone();
                Box::pin(async move { Ok(open) })
            });
        (contracts, contract)
    }

    fn line() -> MaintenanceLineId {
        MaintenanceLineId(Uuid::new_v4())
    }

    #[tokio::test]
    async fn spec_ent_6_each_line_becomes_an_undated_backlog_task_in_the_contract_project() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let lines = vec![line(), line()];
        let (contracts, contract) = contracts_with(organization_id, lines.clone(), HashSet::new());
        let mut tasks = MockTaskRepository::new();
        tasks.expect_insert().times(2).returning(|task| {
            let task = task.clone();
            Box::pin(async move { Ok(task) })
        });
        let mut service = PlanService::new(contracts, tasks);

        let created = service
            .plan(organization_id, &lines, 2026, week(43))
            .await
            .unwrap();

        assert_eq!(created.len(), 2);
        for (task, line_id) in created.iter().zip(&lines) {
            assert_eq!(task.maintenance_line_id, Some(*line_id));
            assert_eq!(task.project_id, Some(contract.project_id));
            assert_eq!(task.customer_id, Some(contract.customer_id));
            assert_eq!(task.customer_context_id, Some(contract.customer_context_id));
            assert_eq!(task.status, TaskStatus::Backlog);
            assert_eq!((task.starts_at, task.ends_at), (None, None));
            assert_eq!(task.organization_id, organization_id);
        }
    }

    #[tokio::test]
    async fn a_line_listed_twice_gets_one_task() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let only = line();
        let (contracts, _) = contracts_with(organization_id, vec![only], HashSet::new());
        let mut tasks = MockTaskRepository::new();
        tasks.expect_insert().times(1).returning(|task| {
            let task = task.clone();
            Box::pin(async move { Ok(task) })
        });
        let mut service = PlanService::new(contracts, tasks);

        let created = service
            .plan(organization_id, &[only, only], 2026, week(43))
            .await
            .unwrap();

        assert_eq!(created.len(), 1);
    }

    #[tokio::test]
    async fn a_line_already_carried_by_an_open_task_is_a_conflict_and_nothing_is_created() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let lines = vec![line(), line()];
        let open = HashSet::from([lines[1]]);
        let (contracts, _) = contracts_with(organization_id, lines.clone(), open);
        let mut tasks = MockTaskRepository::new();
        tasks.expect_insert().never();
        let mut service = PlanService::new(contracts, tasks);

        let result = service.plan(organization_id, &lines, 2026, week(43)).await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn a_line_of_another_organization_is_not_found() {
        let (contracts, _) =
            contracts_with(OrganizationId(Uuid::new_v4()), vec![line()], HashSet::new());
        let mut tasks = MockTaskRepository::new();
        tasks.expect_insert().never();
        let mut service = PlanService::new(contracts, tasks);

        let result = service
            .plan(OrganizationId(Uuid::new_v4()), &[line()], 2026, week(43))
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn an_archived_contract_cannot_be_planned() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let only = line();
        let mut archived = contract(organization_id, &[only]);
        archived.archived_at = Some(Utc::now());
        let mut contracts = MockMaintenanceContractRepository::new();
        contracts.expect_find_by_line().returning(move |_| {
            let archived = archived.clone();
            Box::pin(async move { Ok(Some(archived)) })
        });
        let mut tasks = MockTaskRepository::new();
        tasks.expect_insert().never();
        let mut service = PlanService::new(contracts, tasks);

        let result = service.plan(organization_id, &[only], 2026, week(43)).await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }
}
