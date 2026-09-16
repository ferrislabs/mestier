use common::{CoreError, OrganizationId};
use mestier_macros::transactional;
use uuid::Uuid;

use crate::{
    application::MestierUseCase,
    domain::automation::{
        event_catalogue,
        ports::{SubscriptionRepository, WorkflowRepository},
        subscription::{self, SetWorkflowTriggerCommand, WorkflowTrigger},
        workflow::WorkflowTriggerMode,
    },
};

impl MestierUseCase {
    /// Sets or clears which event(s) trigger a workflow (#225): validates
    /// `command.event_names` against the same catalogue the frontend's
    /// trigger picker lists from, then replaces any existing subscription
    /// for the workflow. Never touches `automation.run` or the dispatcher's
    /// matching logic (`PgEventDispatchRepository::dispatch_pending`) —
    /// this only manages the subscription row it reads.
    #[transactional(workflow, subscription)]
    pub async fn set_workflow_trigger(
        &self,
        command: SetWorkflowTriggerCommand,
    ) -> Result<WorkflowTrigger, CoreError> {
        let mut workflows = workflow_repository;
        let mut subscriptions = subscription_repository;

        workflows
            .find_by_id(command.org_id, command.workflow_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let event_names = subscription::set_workflow_trigger(
            &mut subscriptions,
            &event_catalogue(),
            command.org_id,
            command.workflow_id,
            &command.trigger,
        )
        .await?;

        let mode = match command.trigger {
            WorkflowTrigger::Events(_) => WorkflowTriggerMode::Events,
            WorkflowTrigger::Manual => WorkflowTriggerMode::Manual,
        };
        workflows
            .set_trigger_mode(command.org_id, command.workflow_id, mode)
            .await?;

        Ok(match mode {
            WorkflowTriggerMode::Events => WorkflowTrigger::Events(event_names),
            WorkflowTriggerMode::Manual => WorkflowTrigger::Manual,
        })
    }

    /// The event(s) a workflow currently triggers from, empty when it has no
    /// subscription — an absence, not a 404, per #225's acceptance.
    #[transactional(workflow, subscription)]
    pub async fn workflow_trigger(
        &self,
        org_id: OrganizationId,
        workflow_id: Uuid,
    ) -> Result<WorkflowTrigger, CoreError> {
        let mut workflows = workflow_repository;
        let mut subscriptions = subscription_repository;

        let workflow = workflows
            .find_by_id(org_id, workflow_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        match workflow.trigger_mode {
            WorkflowTriggerMode::Manual => Ok(WorkflowTrigger::Manual),
            WorkflowTriggerMode::Events => {
                let event_names = subscriptions.workflow_trigger(org_id, workflow_id).await?;
                Ok(WorkflowTrigger::Events(event_names))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use common::generate_uuid_v7;
    use sqlx::PgPool;

    use super::*;
    use crate::application::default_authorizer;
    use crate::application::test_support::automation_pool;
    use crate::domain::automation::workflow::CreateWorkflowCommand;
    use crate::infrastructure::realtime::EventHub;

    async fn make_pool() -> PgPool {
        automation_pool().await
    }

    async fn seed_organization(pool: &PgPool, label: &str) -> OrganizationId {
        let owner_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            owner_id,
            format!("owner-{owner_id}@example.com"),
            format!("owner-{owner_id}"),
            "Owner User",
            format!("sub-owner-{owner_id}"),
        )
        .execute(pool)
        .await
        .unwrap();

        let org_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            format!("{label} Org"),
            format!("{label}-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        OrganizationId(org_id)
    }

    fn use_case(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    async fn seed_workflow(usecase: &MestierUseCase, org_id: OrganizationId) -> Uuid {
        usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Trigger use-case test workflow".to_string(),
                description: None,
            })
            .await
            .unwrap()
            .id
    }

    fn events(names: &[&str]) -> WorkflowTrigger {
        WorkflowTrigger::Events(names.iter().map(|n| (*n).to_string()).collect())
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_workflow_with_no_subscription_reads_back_an_empty_trigger() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "trigger-empty").await;
        let usecase = use_case(pool);
        let workflow_id = seed_workflow(&usecase, org_id).await;

        let trigger = usecase.workflow_trigger(org_id, workflow_id).await.unwrap();

        assert_eq!(trigger, WorkflowTrigger::Events(Vec::new()));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn selecting_events_persists_and_is_reflected_on_reload() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "trigger-select").await;
        let usecase = use_case(pool);
        let workflow_id = seed_workflow(&usecase, org_id).await;

        let written = usecase
            .set_workflow_trigger(SetWorkflowTriggerCommand {
                org_id,
                workflow_id,
                trigger: events(&["quote.accepted"]),
            })
            .await
            .unwrap();
        assert_eq!(written, events(&["quote.accepted"]));

        let reloaded = usecase.workflow_trigger(org_id, workflow_id).await.unwrap();
        assert_eq!(reloaded, events(&["quote.accepted"]));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn changing_the_selection_replaces_it() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "trigger-change").await;
        let usecase = use_case(pool.clone());
        let workflow_id = seed_workflow(&usecase, org_id).await;
        usecase
            .set_workflow_trigger(SetWorkflowTriggerCommand {
                org_id,
                workflow_id,
                trigger: events(&["quote.accepted"]),
            })
            .await
            .unwrap();

        let second = usecase
            .set_workflow_trigger(SetWorkflowTriggerCommand {
                org_id,
                workflow_id,
                trigger: events(&["quote.declined"]),
            })
            .await
            .unwrap();

        assert_eq!(second, events(&["quote.declined"]));
        let reloaded = usecase.workflow_trigger(org_id, workflow_id).await.unwrap();
        assert_eq!(reloaded, events(&["quote.declined"]));
        let rows = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM automation.subscription
               WHERE kind = 'workflow' AND target_id = $1"#,
            workflow_id,
        )
        .fetch_one(&pool)
        .await
        .unwrap()
        .unwrap_or(0);
        assert_eq!(rows, 1, "changing the selection must not accumulate rows");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn clearing_the_selection_removes_the_subscription() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "trigger-clear").await;
        let usecase = use_case(pool);
        let workflow_id = seed_workflow(&usecase, org_id).await;
        usecase
            .set_workflow_trigger(SetWorkflowTriggerCommand {
                org_id,
                workflow_id,
                trigger: events(&["quote.accepted"]),
            })
            .await
            .unwrap();

        let cleared = usecase
            .set_workflow_trigger(SetWorkflowTriggerCommand {
                org_id,
                workflow_id,
                trigger: events(&[]),
            })
            .await
            .unwrap();

        assert_eq!(cleared, events(&[]));
        let reloaded = usecase.workflow_trigger(org_id, workflow_id).await.unwrap();
        assert_eq!(reloaded, events(&[]));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_event_name_outside_the_catalogue_is_rejected_and_nothing_is_stored() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "trigger-invalid").await;
        let usecase = use_case(pool);
        let workflow_id = seed_workflow(&usecase, org_id).await;

        let error = usecase
            .set_workflow_trigger(SetWorkflowTriggerCommand {
                org_id,
                workflow_id,
                trigger: events(&["not.a.real.event"]),
            })
            .await
            .expect_err("an unknown event name must be refused");

        assert!(matches!(error, CoreError::Conflict(_)));
        let trigger = usecase.workflow_trigger(org_id, workflow_id).await.unwrap();
        assert_eq!(trigger, events(&[]));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn setting_a_trigger_for_a_missing_workflow_is_not_found() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "trigger-missing").await;
        let usecase = use_case(pool);

        let error = usecase
            .set_workflow_trigger(SetWorkflowTriggerCommand {
                org_id,
                workflow_id: generate_uuid_v7(),
                trigger: events(&["quote.accepted"]),
            })
            .await
            .expect_err("there is no workflow to trigger");

        assert!(matches!(error, CoreError::NotFound));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn reading_another_organizations_workflow_trigger_is_not_found() {
        let pool = make_pool().await;
        let owner_org = seed_organization(&pool, "trigger-owner").await;
        let stranger_org = seed_organization(&pool, "trigger-stranger").await;
        let usecase = use_case(pool);
        let workflow_id = seed_workflow(&usecase, owner_org).await;
        usecase
            .set_workflow_trigger(SetWorkflowTriggerCommand {
                org_id: owner_org,
                workflow_id,
                trigger: events(&["quote.accepted"]),
            })
            .await
            .unwrap();

        let error = usecase
            .workflow_trigger(stranger_org, workflow_id)
            .await
            .expect_err("a stranger's workflow must read back as absent, not as data");

        assert!(matches!(error, CoreError::NotFound));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn switching_to_manual_clears_the_subscription_and_reads_back_manual() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "trigger-manual").await;
        let usecase = use_case(pool.clone());
        let workflow_id = seed_workflow(&usecase, org_id).await;
        usecase
            .set_workflow_trigger(SetWorkflowTriggerCommand {
                org_id,
                workflow_id,
                trigger: events(&["quote.accepted"]),
            })
            .await
            .unwrap();

        let written = usecase
            .set_workflow_trigger(SetWorkflowTriggerCommand {
                org_id,
                workflow_id,
                trigger: WorkflowTrigger::Manual,
            })
            .await
            .unwrap();

        assert_eq!(written, WorkflowTrigger::Manual);
        let reloaded = usecase.workflow_trigger(org_id, workflow_id).await.unwrap();
        assert_eq!(reloaded, WorkflowTrigger::Manual);
        let rows = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM automation.subscription
               WHERE kind = 'workflow' AND target_id = $1"#,
            workflow_id,
        )
        .fetch_one(&pool)
        .await
        .unwrap()
        .unwrap_or(0);
        assert_eq!(
            rows, 0,
            "a manual workflow must have no subscription row at all"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn switching_manual_to_events_and_back_leaves_no_orphan_subscription() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "trigger-round-trip").await;
        let usecase = use_case(pool.clone());
        let workflow_id = seed_workflow(&usecase, org_id).await;

        usecase
            .set_workflow_trigger(SetWorkflowTriggerCommand {
                org_id,
                workflow_id,
                trigger: WorkflowTrigger::Manual,
            })
            .await
            .unwrap();
        usecase
            .set_workflow_trigger(SetWorkflowTriggerCommand {
                org_id,
                workflow_id,
                trigger: events(&["quote.accepted"]),
            })
            .await
            .unwrap();
        usecase
            .set_workflow_trigger(SetWorkflowTriggerCommand {
                org_id,
                workflow_id,
                trigger: WorkflowTrigger::Manual,
            })
            .await
            .unwrap();

        let reloaded = usecase.workflow_trigger(org_id, workflow_id).await.unwrap();
        assert_eq!(reloaded, WorkflowTrigger::Manual);
        let rows = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM automation.subscription
               WHERE kind = 'workflow' AND target_id = $1"#,
            workflow_id,
        )
        .fetch_one(&pool)
        .await
        .unwrap()
        .unwrap_or(0);
        assert_eq!(rows, 0, "the round trip must leave no orphan subscription");
    }
}
