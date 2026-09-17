use common::{CoreError, OrganizationId, generate_uuid_v7};
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    domain::automation::{
        ports::SubscriptionRepository,
        workflow::{PlacedTrigger, TriggerKind},
    },
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = Subscription, backend = Postgres)]
pub struct PgSubscriptionRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgSubscriptionRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> SubscriptionRepository for PgSubscriptionRepository<'tx> {
    async fn replace_workflow_subscriptions(
        &mut self,
        org_id: OrganizationId,
        workflow_id: Uuid,
        triggers: &[PlacedTrigger],
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;

        sqlx::query!(
            r#"
            DELETE FROM automation.subscription
            WHERE org_id = $1 AND kind = 'workflow' AND target_id = $2
            "#,
            org_id.0,
            workflow_id,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        for trigger in triggers {
            let TriggerKind::Events(event_names) = &trigger.kind else {
                continue;
            };

            sqlx::query!(
                r#"
                INSERT INTO automation.subscription
                    (id, org_id, kind, target_id, trigger_id, event_names, enabled)
                VALUES ($1, $2, 'workflow', $3, $4, $5, true)
                "#,
                generate_uuid_v7(),
                org_id.0,
                workflow_id,
                trigger.id,
                event_names,
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;
    use crate::application::test_support::automation_pool;
    use crate::infrastructure::postgres::with_tx;

    async fn make_pool() -> PgPool {
        automation_pool().await
    }

    async fn seed_organization(pool: &PgPool) -> OrganizationId {
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
            "Test Org",
            format!("test-org-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        OrganizationId(org_id)
    }

    async fn seed_workflow(pool: &PgPool, org_id: OrganizationId) -> Uuid {
        let usecase = crate::application::MestierUseCase::new(
            pool.clone(),
            crate::application::default_authorizer(),
            crate::infrastructure::realtime::EventHub::new(),
        );
        let workflow = usecase
            .create_workflow(crate::domain::automation::workflow::CreateWorkflowCommand {
                org_id,
                name: "Subscription test workflow".to_string(),
                description: None,
            })
            .await
            .unwrap();
        workflow.id
    }

    async fn subscription_rows(
        pool: &PgPool,
        workflow_id: Uuid,
    ) -> Vec<(Option<String>, Vec<String>)> {
        let rows = sqlx::query!(
            r#"SELECT trigger_id, event_names FROM automation.subscription
               WHERE kind = 'workflow' AND target_id = $1
               ORDER BY trigger_id"#,
            workflow_id,
        )
        .fetch_all(pool)
        .await
        .unwrap();

        rows.into_iter()
            .map(|row| (row.trigger_id, row.event_names))
            .collect()
    }

    fn events_trigger(id: &str, names: &[&str]) -> PlacedTrigger {
        PlacedTrigger {
            id: id.to_string(),
            kind: TriggerKind::Events(names.iter().map(|n| (*n).to_string()).collect()),
        }
    }

    fn manual_trigger(id: &str) -> PlacedTrigger {
        PlacedTrigger {
            id: id.to_string(),
            kind: TriggerKind::Manual,
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn two_event_triggers_write_two_subscription_rows() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;
        let triggers = vec![
            events_trigger("t1", &["quote.accepted"]),
            events_trigger("t2", &["quote.declined", "invoice.issued"]),
        ];

        with_tx(&pool, async |tx| {
            let mut repo = PgSubscriptionRepository::new(&tx);
            repo.replace_workflow_subscriptions(org_id, workflow_id, &triggers)
                .await
        })
        .await
        .unwrap();

        let rows = subscription_rows(&pool, workflow_id).await;
        assert_eq!(
            rows,
            vec![
                (Some("t1".to_string()), vec!["quote.accepted".to_string()]),
                (
                    Some("t2".to_string()),
                    vec!["quote.declined".to_string(), "invoice.issued".to_string()]
                ),
            ]
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_manual_trigger_gets_no_subscription_row() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;
        let triggers = vec![manual_trigger("t1")];

        with_tx(&pool, async |tx| {
            let mut repo = PgSubscriptionRepository::new(&tx);
            repo.replace_workflow_subscriptions(org_id, workflow_id, &triggers)
                .await
        })
        .await
        .unwrap();

        assert!(subscription_rows(&pool, workflow_id).await.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn removing_a_trigger_and_replacing_removes_its_row() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;
        with_tx(&pool, async |tx| {
            let mut repo = PgSubscriptionRepository::new(&tx);
            repo.replace_workflow_subscriptions(
                org_id,
                workflow_id,
                &[
                    events_trigger("t1", &["quote.accepted"]),
                    events_trigger("t2", &["quote.declined"]),
                ],
            )
            .await
        })
        .await
        .unwrap();

        with_tx(&pool, async |tx| {
            let mut repo = PgSubscriptionRepository::new(&tx);
            repo.replace_workflow_subscriptions(
                org_id,
                workflow_id,
                &[events_trigger("t1", &["quote.accepted"])],
            )
            .await
        })
        .await
        .unwrap();

        let rows = subscription_rows(&pool, workflow_id).await;
        assert_eq!(
            rows,
            vec![(Some("t1".to_string()), vec!["quote.accepted".to_string()])]
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn replacing_with_no_triggers_clears_every_row() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;
        with_tx(&pool, async |tx| {
            let mut repo = PgSubscriptionRepository::new(&tx);
            repo.replace_workflow_subscriptions(
                org_id,
                workflow_id,
                &[events_trigger("t1", &["quote.accepted"])],
            )
            .await
        })
        .await
        .unwrap();

        with_tx(&pool, async |tx| {
            let mut repo = PgSubscriptionRepository::new(&tx);
            repo.replace_workflow_subscriptions(org_id, workflow_id, &[])
                .await
        })
        .await
        .unwrap();

        assert!(subscription_rows(&pool, workflow_id).await.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn another_organizations_subscriptions_are_untouched() {
        let pool = make_pool().await;
        let mine = seed_organization(&pool).await;
        let theirs = seed_organization(&pool).await;
        let their_workflow = seed_workflow(&pool, theirs).await;
        with_tx(&pool, async |tx| {
            let mut repo = PgSubscriptionRepository::new(&tx);
            repo.replace_workflow_subscriptions(
                theirs,
                their_workflow,
                &[events_trigger("t1", &["quote.accepted"])],
            )
            .await
        })
        .await
        .unwrap();

        with_tx(&pool, async |tx| {
            let mut repo = PgSubscriptionRepository::new(&tx);
            repo.replace_workflow_subscriptions(mine, their_workflow, &[])
                .await
        })
        .await
        .unwrap();

        let rows = subscription_rows(&pool, their_workflow).await;
        assert_eq!(
            rows,
            vec![(Some("t1".to_string()), vec!["quote.accepted".to_string()])],
            "a stranger's replace call must not touch another organization's rows"
        );
    }
}
