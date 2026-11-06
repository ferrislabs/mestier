use common::{CoreError, OrganizationId};
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    domain::automation::{
        ports::WebhookRouteRepository,
        workflow::{PlacedTrigger, ResolvedWebhook, TriggerKind, WebhookMethod},
    },
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

pub const WEBHOOK_PATH_CONSTRAINT: &str = "uq_automation_webhook_route_path";

#[repository(domain = WebhookRoute, backend = Postgres)]
pub struct PgWebhookRouteRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgWebhookRouteRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> WebhookRouteRepository for PgWebhookRouteRepository<'tx> {
    async fn replace_workflow_routes(
        &mut self,
        org_id: OrganizationId,
        workflow_id: Uuid,
        triggers: &[PlacedTrigger],
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;

        sqlx::query!(
            "DELETE FROM automation.webhook_route WHERE org_id = $1 AND workflow_id = $2",
            org_id.0,
            workflow_id,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        for trigger in triggers {
            let TriggerKind::Webhook { method, path, .. } = &trigger.kind else {
                continue;
            };

            sqlx::query!(
                r#"
                INSERT INTO automation.webhook_route (path, org_id, workflow_id, trigger_id, method)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                path,
                org_id.0,
                workflow_id,
                trigger.id,
                method.as_str(),
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        Ok(())
    }

    async fn paths_held_by_other_workflows(
        &mut self,
        workflow_id: Uuid,
        paths: &[String],
    ) -> Result<Vec<String>, CoreError> {
        let mut tx = self.tx.lock().await;

        sqlx::query_scalar!(
            r#"
            SELECT path
            FROM automation.webhook_route
            WHERE path = ANY($1) AND workflow_id <> $2
            ORDER BY path
            "#,
            paths,
            workflow_id,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)
    }

    async fn resolve(
        &mut self,
        path: &str,
        method: WebhookMethod,
    ) -> Result<Option<ResolvedWebhook>, CoreError> {
        let mut tx = self.tx.lock().await;

        let row = sqlx::query!(
            r#"
            SELECT r.org_id, r.workflow_id, trig.value AS "trigger!"
            FROM automation.webhook_route r
            JOIN automation.workflow w
              ON w.id = r.workflow_id
             AND w.org_id = r.org_id
             AND w.enabled
             AND w.archived_at IS NULL
            JOIN automation.workflow_version wv ON wv.id = w.current_version_id
            CROSS JOIN LATERAL jsonb_array_elements(wv.graph -> 'triggers') AS trig(value)
            WHERE r.path = $1
              AND r.method = $2
              AND trig.value ->> 'id' = r.trigger_id
            LIMIT 1
            "#,
            path,
            method.as_str(),
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        let Ok(trigger) = serde_json::from_value::<PlacedTrigger>(row.trigger) else {
            tracing::error!(workflow_id = %row.workflow_id, "a webhook trigger could not be read from its graph");
            return Ok(None);
        };

        Ok(ResolvedWebhook::from_trigger(
            OrganizationId(row.org_id),
            row.workflow_id,
            trigger,
        ))
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;
    use crate::application::automation::trigger_fixtures::{
        archive, graph_of, key, seed_organization, seed_workflow, set_enabled, webhook,
        webhook_with,
    };
    use crate::application::test_support::automation_pool;
    use crate::domain::automation::workflow::{ContractKind, OnInvalid, TriggerKind};
    use crate::infrastructure::postgres::with_tx;

    async fn replace(
        pool: &PgPool,
        org_id: OrganizationId,
        workflow_id: Uuid,
        triggers: &[PlacedTrigger],
    ) -> Result<(), CoreError> {
        with_tx(pool, async |tx| {
            PgWebhookRouteRepository::new(&tx)
                .replace_workflow_routes(org_id, workflow_id, triggers)
                .await
        })
        .await
    }

    async fn resolve(pool: &PgPool, path: &str, method: WebhookMethod) -> Option<ResolvedWebhook> {
        with_tx(pool, async |tx| {
            PgWebhookRouteRepository::new(&tx)
                .resolve(path, method)
                .await
        })
        .await
        .unwrap()
    }

    async fn rows(pool: &PgPool, workflow_id: Uuid) -> Vec<(String, String, String)> {
        sqlx::query!(
            r#"SELECT path, trigger_id, method FROM automation.webhook_route
               WHERE workflow_id = $1 ORDER BY trigger_id"#,
            workflow_id,
        )
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row| (row.path, row.trigger_id, row.method))
        .collect()
    }

    fn unique(label: &str) -> String {
        format!("{label}-{}", Uuid::now_v7().simple())
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_route_is_written_for_each_webhook_trigger_and_for_nothing_else() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "routes").await;
        let (first, second) = (unique("a"), unique("b"));
        let graph = graph_of(vec![
            ("t1", webhook(WebhookMethod::Post, &first)),
            ("t2", TriggerKind::Manual),
            ("t3", webhook(WebhookMethod::Get, &second)),
            (
                "t4",
                TriggerKind::Events(vec!["quote.accepted".to_string()]),
            ),
        ]);
        let workflow_id = seed_workflow(&pool, org_id, &graph).await;

        replace(&pool, org_id, workflow_id, &graph.triggers)
            .await
            .unwrap();

        assert_eq!(
            rows(&pool, workflow_id).await,
            vec![
                (first, "t1".to_string(), "POST".to_string()),
                (second, "t3".to_string(), "GET".to_string()),
            ]
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn saving_again_replaces_the_routes_of_the_workflow() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "routes-replace").await;
        let (old, new) = (unique("old"), unique("new"));
        let before = graph_of(vec![("t1", webhook(WebhookMethod::Post, &old))]);
        let workflow_id = seed_workflow(&pool, org_id, &before).await;
        replace(&pool, org_id, workflow_id, &before.triggers)
            .await
            .unwrap();

        let after = graph_of(vec![("t1", webhook(WebhookMethod::Put, &new))]);
        replace(&pool, org_id, workflow_id, &after.triggers)
            .await
            .unwrap();

        assert_eq!(
            rows(&pool, workflow_id).await,
            vec![(new, "t1".to_string(), "PUT".to_string())]
        );
        assert!(resolve(&pool, &old, WebhookMethod::Post).await.is_none());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_workflow_can_keep_its_own_path_across_saves() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "routes-keep").await;
        let path = unique("keep");
        let graph = graph_of(vec![("t1", webhook(WebhookMethod::Post, &path))]);
        let workflow_id = seed_workflow(&pool, org_id, &graph).await;

        for _ in 0..2 {
            replace(&pool, org_id, workflow_id, &graph.triggers)
                .await
                .unwrap();
        }

        assert_eq!(rows(&pool, workflow_id).await.len(), 1);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_second_workflow_cannot_take_a_path_even_from_another_organization() {
        let pool = automation_pool().await;
        let first_org = seed_organization(&pool, "routes-first").await;
        let second_org = seed_organization(&pool, "routes-second").await;
        let path = unique("shared");
        let graph = graph_of(vec![("t1", webhook(WebhookMethod::Post, &path))]);
        let first = seed_workflow(&pool, first_org, &graph).await;
        let second = seed_workflow(&pool, second_org, &graph).await;
        replace(&pool, first_org, first, &graph.triggers)
            .await
            .unwrap();

        let refused = replace(&pool, second_org, second, &graph.triggers).await;

        assert!(
            matches!(&refused, Err(CoreError::Conflict(constraint)) if constraint == "uq_automation_webhook_route_path"),
            "{refused:?}"
        );
        assert!(rows(&pool, second).await.is_empty());
        assert_eq!(rows(&pool, first).await.len(), 1);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_paths_other_workflows_hold_are_reported() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "routes-held").await;
        let (taken, mine, free) = (unique("taken"), unique("mine"), unique("free"));
        let other_graph = graph_of(vec![("t1", webhook(WebhookMethod::Post, &taken))]);
        let other = seed_workflow(&pool, org_id, &other_graph).await;
        replace(&pool, org_id, other, &other_graph.triggers)
            .await
            .unwrap();
        let my_graph = graph_of(vec![("t1", webhook(WebhookMethod::Post, &mine))]);
        let workflow_id = seed_workflow(&pool, org_id, &my_graph).await;
        replace(&pool, org_id, workflow_id, &my_graph.triggers)
            .await
            .unwrap();

        let held = with_tx(&pool, async |tx| {
            PgWebhookRouteRepository::new(&tx)
                .paths_held_by_other_workflows(
                    workflow_id,
                    &[taken.clone(), mine.clone(), free.clone()],
                )
                .await
        })
        .await
        .unwrap();

        assert_eq!(held, vec![taken]);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_route_resolves_to_the_trigger_it_was_derived_from() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "routes-resolve").await;
        let path = unique("resolve");
        let credential_id = Uuid::now_v7();
        let graph = graph_of(vec![(
            "t1",
            webhook_with(
                &path,
                Some(credential_id),
                vec![key("montant", ContractKind::Number, true)],
                OnInvalid::Accept,
            ),
        )]);
        let workflow_id = seed_workflow(&pool, org_id, &graph).await;
        replace(&pool, org_id, workflow_id, &graph.triggers)
            .await
            .unwrap();

        let resolved = resolve(&pool, &path, WebhookMethod::Post).await;

        assert_eq!(
            resolved,
            Some(ResolvedWebhook {
                org_id,
                workflow_id,
                trigger_id: "t1".to_string(),
                credential_id: Some(credential_id),
                contract: vec![key("montant", ContractKind::Number, true)],
                on_invalid: OnInvalid::Accept,
            })
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_wrong_method_or_an_unknown_path_resolves_to_nothing() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "routes-method").await;
        let path = unique("method");
        let graph = graph_of(vec![("t1", webhook(WebhookMethod::Post, &path))]);
        let workflow_id = seed_workflow(&pool, org_id, &graph).await;
        replace(&pool, org_id, workflow_id, &graph.triggers)
            .await
            .unwrap();

        for method in [
            WebhookMethod::Get,
            WebhookMethod::Put,
            WebhookMethod::Patch,
            WebhookMethod::Delete,
        ] {
            assert!(resolve(&pool, &path, method).await.is_none(), "{method:?}");
        }
        assert!(
            resolve(&pool, &unique("unknown"), WebhookMethod::Post)
                .await
                .is_none()
        );
        assert!(resolve(&pool, &path, WebhookMethod::Post).await.is_some());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_disabled_or_archived_workflow_resolves_to_nothing_and_comes_back_when_enabled() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "routes-disabled").await;
        let path = unique("disabled");
        let graph = graph_of(vec![("t1", webhook(WebhookMethod::Post, &path))]);
        let workflow_id = seed_workflow(&pool, org_id, &graph).await;
        replace(&pool, org_id, workflow_id, &graph.triggers)
            .await
            .unwrap();
        assert!(resolve(&pool, &path, WebhookMethod::Post).await.is_some());

        set_enabled(&pool, workflow_id, false).await;
        assert!(resolve(&pool, &path, WebhookMethod::Post).await.is_none());

        set_enabled(&pool, workflow_id, true).await;
        assert!(resolve(&pool, &path, WebhookMethod::Post).await.is_some());

        archive(&pool, workflow_id).await;
        assert!(resolve(&pool, &path, WebhookMethod::Post).await.is_none());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_route_whose_trigger_is_no_longer_a_webhook_in_the_current_version_resolves_to_nothing()
     {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "routes-drift").await;
        let path = unique("drift");
        let graph = graph_of(vec![("t1", TriggerKind::Manual)]);
        let workflow_id = seed_workflow(&pool, org_id, &graph).await;
        sqlx::query!(
            "INSERT INTO automation.webhook_route (path, org_id, workflow_id, trigger_id, method)
             VALUES ($1, $2, $3, 't1', 'POST')",
            path,
            org_id.0,
            workflow_id,
        )
        .execute(&pool)
        .await
        .unwrap();

        assert!(resolve(&pool, &path, WebhookMethod::Post).await.is_none());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_the_workflow_removes_its_routes() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "routes-delete").await;
        let path = unique("delete");
        let graph = graph_of(vec![("t1", webhook(WebhookMethod::Post, &path))]);
        let workflow_id = seed_workflow(&pool, org_id, &graph).await;
        replace(&pool, org_id, workflow_id, &graph.triggers)
            .await
            .unwrap();
        sqlx::query!(
            "UPDATE automation.workflow SET current_version_id = NULL WHERE id = $1",
            workflow_id
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!("DELETE FROM automation.workflow WHERE id = $1", workflow_id)
            .execute(&pool)
            .await
            .unwrap();

        assert!(rows(&pool, workflow_id).await.is_empty());
    }
}
