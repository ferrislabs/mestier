use common::{CoreError, OrganizationId, generate_uuid_v7};
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    domain::automation::ports::SubscriptionRepository,
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

/// A workflow's trigger, stored as `automation.subscription` rows of
/// `kind = 'workflow'` — the same table and identity
/// `PgEventDispatchRepository::dispatch_pending` already reads; this is
/// just the write side of it, plus the single-workflow read the trigger
/// picker needs.
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
    async fn set_workflow_trigger(
        &mut self,
        org_id: OrganizationId,
        workflow_id: Uuid,
        event_names: &[String],
    ) -> Result<Vec<String>, CoreError> {
        let mut tx = self.tx.lock().await;

        // Delete-then-insert rather than an `ON CONFLICT` upsert: there is
        // no unique constraint on `(org_id, kind, target_id)` to conflict
        // against — a workflow's subscription is identified by the pair
        // `kind = 'workflow'` and `target_id`, and this pair of statements
        // is what keeps it a single row rather than an accumulation of one
        // per edit. Both run in the same transaction the `#[transactional]`
        // use case already opened, so a reader never observes the row
        // gone-and-not-yet-replaced.
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

        if event_names.is_empty() {
            // A workflow triggered by nothing has no subscription row at
            // all — not a row subscribed to an empty set of events, which
            // `chk_automation_subscription_event_names` would refuse to
            // store anyway.
            return Ok(Vec::new());
        }

        let row = sqlx::query!(
            r#"
            INSERT INTO automation.subscription (id, org_id, kind, target_id, event_names, enabled)
            VALUES ($1, $2, 'workflow', $3, $4, true)
            RETURNING event_names
            "#,
            generate_uuid_v7(),
            org_id.0,
            workflow_id,
            event_names,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.event_names)
    }

    async fn workflow_trigger(
        &mut self,
        org_id: OrganizationId,
        workflow_id: Uuid,
    ) -> Result<Vec<String>, CoreError> {
        let mut tx = self.tx.lock().await;

        let row = sqlx::query!(
            r#"
            SELECT event_names
            FROM automation.subscription
            WHERE org_id = $1 AND kind = 'workflow' AND target_id = $2
            "#,
            org_id.0,
            workflow_id,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(|r| r.event_names).unwrap_or_default())
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
                name: "Trigger test workflow".to_string(),
                description: None,
            })
            .await
            .unwrap();
        workflow.id
    }

    async fn subscription_row_count(pool: &PgPool, workflow_id: Uuid) -> i64 {
        sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM automation.subscription
               WHERE kind = 'workflow' AND target_id = $1"#,
            workflow_id,
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .unwrap_or(0)
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_workflow_with_no_subscription_reads_back_an_empty_selection() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;

        let trigger = with_tx(&pool, async |tx| {
            let mut repo = PgSubscriptionRepository::new(&tx);
            repo.workflow_trigger(org_id, workflow_id).await
        })
        .await
        .unwrap();

        assert!(trigger.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn setting_a_trigger_persists_it_and_is_read_back() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;

        let written = with_tx(&pool, async |tx| {
            let mut repo = PgSubscriptionRepository::new(&tx);
            repo.set_workflow_trigger(
                org_id,
                workflow_id,
                &["quote.accepted".to_string(), "invoice.issued".to_string()],
            )
            .await
        })
        .await
        .unwrap();
        assert_eq!(
            written,
            vec!["quote.accepted".to_string(), "invoice.issued".to_string()]
        );

        let reread = with_tx(&pool, async |tx| {
            let mut repo = PgSubscriptionRepository::new(&tx);
            repo.workflow_trigger(org_id, workflow_id).await
        })
        .await
        .unwrap();
        assert_eq!(
            reread,
            vec!["quote.accepted".to_string(), "invoice.issued".to_string()]
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn changing_the_selection_replaces_it_rather_than_accumulating_a_second_row() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;

        with_tx(&pool, async |tx| {
            let mut repo = PgSubscriptionRepository::new(&tx);
            repo.set_workflow_trigger(org_id, workflow_id, &["quote.accepted".to_string()])
                .await
        })
        .await
        .unwrap();

        with_tx(&pool, async |tx| {
            let mut repo = PgSubscriptionRepository::new(&tx);
            repo.set_workflow_trigger(org_id, workflow_id, &["quote.declined".to_string()])
                .await
        })
        .await
        .unwrap();

        assert_eq!(
            subscription_row_count(&pool, workflow_id).await,
            1,
            "the second write must replace the first row, not add to it"
        );
        let reread = with_tx(&pool, async |tx| {
            let mut repo = PgSubscriptionRepository::new(&tx);
            repo.workflow_trigger(org_id, workflow_id).await
        })
        .await
        .unwrap();
        assert_eq!(reread, vec!["quote.declined".to_string()]);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn clearing_the_selection_removes_the_row_entirely() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;
        with_tx(&pool, async |tx| {
            let mut repo = PgSubscriptionRepository::new(&tx);
            repo.set_workflow_trigger(org_id, workflow_id, &["quote.accepted".to_string()])
                .await
        })
        .await
        .unwrap();

        with_tx(&pool, async |tx| {
            let mut repo = PgSubscriptionRepository::new(&tx);
            repo.set_workflow_trigger(org_id, workflow_id, &[]).await
        })
        .await
        .unwrap();

        assert_eq!(subscription_row_count(&pool, workflow_id).await, 0);
        let reread = with_tx(&pool, async |tx| {
            let mut repo = PgSubscriptionRepository::new(&tx);
            repo.workflow_trigger(org_id, workflow_id).await
        })
        .await
        .unwrap();
        assert!(reread.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn another_organizations_trigger_is_not_visible() {
        let pool = make_pool().await;
        let mine = seed_organization(&pool).await;
        let theirs = seed_organization(&pool).await;
        let their_workflow = seed_workflow(&pool, theirs).await;
        with_tx(&pool, async |tx| {
            let mut repo = PgSubscriptionRepository::new(&tx);
            repo.set_workflow_trigger(theirs, their_workflow, &["quote.accepted".to_string()])
                .await
        })
        .await
        .unwrap();

        let trigger = with_tx(&pool, async |tx| {
            let mut repo = PgSubscriptionRepository::new(&tx);
            repo.workflow_trigger(mine, their_workflow).await
        })
        .await
        .unwrap();

        assert!(trigger.is_empty());
    }
}
