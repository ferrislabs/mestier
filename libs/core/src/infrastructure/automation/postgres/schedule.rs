use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId};
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    domain::automation::{
        ports::ScheduleRepository,
        workflow::{DueSchedule, PlacedTrigger, ScheduleSeed},
    },
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = Schedule, backend = Postgres)]
pub struct PgScheduleRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgScheduleRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> ScheduleRepository for PgScheduleRepository<'tx> {
    async fn replace_workflow_schedules(
        &mut self,
        org_id: OrganizationId,
        workflow_id: Uuid,
        schedules: &[ScheduleSeed],
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;

        for schedule in schedules {
            sqlx::query!(
                r#"
                INSERT INTO automation.schedule_state (workflow_id, trigger_id, org_id, next_run_at)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (workflow_id, trigger_id)
                DO UPDATE SET next_run_at = EXCLUDED.next_run_at
                "#,
                workflow_id,
                schedule.trigger_id,
                org_id.0,
                schedule.next_run_at,
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        let kept: Vec<String> = schedules
            .iter()
            .map(|schedule| schedule.trigger_id.clone())
            .collect();
        sqlx::query!(
            r#"
            DELETE FROM automation.schedule_state
            WHERE org_id = $1 AND workflow_id = $2 AND trigger_id <> ALL($3)
            "#,
            org_id.0,
            workflow_id,
            &kept,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn count_other_schedules(
        &mut self,
        org_id: OrganizationId,
        workflow_id: Uuid,
    ) -> Result<i64, CoreError> {
        let mut tx = self.tx.lock().await;

        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
            .bind(format!("automation.trigger_quota:{}", org_id.0))
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;

        sqlx::query_scalar!(
            r#"
            SELECT count(*) AS "count!"
            FROM automation.schedule_state
            WHERE org_id = $1 AND workflow_id <> $2
            "#,
            org_id.0,
            workflow_id,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)
    }

    async fn open_attempt(&mut self) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query("SAVEPOINT schedule_attempt")
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn keep_attempt(&mut self) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query("RELEASE SAVEPOINT schedule_attempt")
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn drop_attempt(&mut self) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query("ROLLBACK TO SAVEPOINT schedule_attempt")
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        sqlx::query("RELEASE SAVEPOINT schedule_attempt")
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn claim_due(
        &mut self,
        now: DateTime<Utc>,
        batch: i64,
        per_org: i64,
    ) -> Result<Vec<DueSchedule>, CoreError> {
        let mut tx = self.tx.lock().await;

        let rows = sqlx::query!(
            r#"
            WITH eligible AS (
                SELECT s.workflow_id, s.trigger_id, s.next_run_at,
                       row_number() OVER (
                           PARTITION BY s.org_id
                           ORDER BY s.next_run_at, s.workflow_id, s.trigger_id
                       ) AS rank
                FROM automation.schedule_state s
                JOIN automation.workflow w
                  ON w.id = s.workflow_id
                 AND w.org_id = s.org_id
                 AND w.enabled
                 AND w.archived_at IS NULL
                 AND w.current_version_id IS NOT NULL
                WHERE s.next_run_at <= $1
            ),
            chosen AS (
                SELECT workflow_id, trigger_id, next_run_at
                FROM eligible
                WHERE rank <= $3
                ORDER BY next_run_at, workflow_id, trigger_id
                LIMIT $2
            )
            SELECT s.org_id,
                   s.workflow_id,
                   s.trigger_id,
                   s.next_run_at,
                   w.current_version_id AS "workflow_version_id!",
                   trig.value AS "trigger!",
                   o.timezone
            FROM chosen c
            JOIN automation.schedule_state s
              ON s.workflow_id = c.workflow_id AND s.trigger_id = c.trigger_id
            JOIN automation.workflow w
              ON w.id = s.workflow_id
             AND w.org_id = s.org_id
             AND w.enabled
             AND w.archived_at IS NULL
             AND w.current_version_id IS NOT NULL
            JOIN automation.workflow_version wv ON wv.id = w.current_version_id
            JOIN organizations o ON o.id = s.org_id
            CROSS JOIN LATERAL jsonb_array_elements(wv.graph -> 'triggers') AS trig(value)
            WHERE s.next_run_at <= $1
              AND trig.value ->> 'id' = s.trigger_id
            ORDER BY s.next_run_at, s.workflow_id, s.trigger_id
            FOR UPDATE OF s SKIP LOCKED
            FOR SHARE OF w SKIP LOCKED
            "#,
            now,
            batch,
            per_org,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                let trigger = match serde_json::from_value::<PlacedTrigger>(row.trigger) {
                    Ok(trigger) => trigger,
                    Err(error) => {
                        tracing::error!(%error, workflow_id = %row.workflow_id, "a schedule trigger could not be read from its graph");
                        return None;
                    }
                };
                Some(DueSchedule {
                    org_id: OrganizationId(row.org_id),
                    workflow_id: row.workflow_id,
                    workflow_version_id: row.workflow_version_id,
                    trigger_id: row.trigger_id,
                    scheduled_for: row.next_run_at,
                    trigger: trigger.kind,
                    organization_timezone: row.timezone,
                })
            })
            .collect())
    }

    async fn advance(
        &mut self,
        workflow_id: Uuid,
        trigger_id: &str,
        last_run_at: Option<DateTime<Utc>>,
        next_run_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;

        sqlx::query!(
            r#"
            UPDATE automation.schedule_state
            SET next_run_at = $3,
                last_run_at = COALESCE($4, last_run_at)
            WHERE workflow_id = $1 AND trigger_id = $2
            "#,
            workflow_id,
            trigger_id,
            next_run_at,
            last_run_at,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use sqlx::PgPool;

    use super::*;
    use crate::application::automation::trigger_fixtures::{
        archive, daily, graph_of, seed_organization, seed_workflow, set_enabled,
        set_organization_timezone,
    };
    use crate::application::test_support::{automation_pool, now_storable};
    use crate::domain::automation::workflow::TriggerKind;
    use crate::infrastructure::postgres::with_tx;

    fn seed(trigger_id: &str, next_run_at: DateTime<Utc>) -> ScheduleSeed {
        ScheduleSeed {
            trigger_id: trigger_id.to_string(),
            next_run_at,
        }
    }

    async fn replace(
        pool: &PgPool,
        org_id: OrganizationId,
        workflow_id: Uuid,
        seeds: &[ScheduleSeed],
    ) {
        with_tx(pool, async |tx| {
            PgScheduleRepository::new(&tx)
                .replace_workflow_schedules(org_id, workflow_id, seeds)
                .await
        })
        .await
        .unwrap();
    }

    async fn claim(pool: &PgPool, now: DateTime<Utc>, batch: i64) -> Vec<DueSchedule> {
        with_tx(pool, async |tx| {
            PgScheduleRepository::new(&tx)
                .claim_due(now, batch, 1000)
                .await
        })
        .await
        .unwrap()
    }

    async fn claim_of(pool: &PgPool, workflow_id: Uuid, now: DateTime<Utc>) -> Vec<DueSchedule> {
        claim(pool, now, 1000)
            .await
            .into_iter()
            .filter(|due| due.workflow_id == workflow_id)
            .collect()
    }

    async fn rows(
        pool: &PgPool,
        workflow_id: Uuid,
    ) -> Vec<(String, DateTime<Utc>, Option<DateTime<Utc>>)> {
        sqlx::query!(
            r#"SELECT trigger_id, next_run_at, last_run_at FROM automation.schedule_state
               WHERE workflow_id = $1 ORDER BY trigger_id"#,
            workflow_id,
        )
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row| (row.trigger_id, row.next_run_at, row.last_run_at))
        .collect()
    }

    async fn scheduled_workflow(pool: &PgPool, label: &str) -> (OrganizationId, Uuid) {
        let org_id = seed_organization(pool, label).await;
        let graph = graph_of(vec![
            ("t1", daily("09:00", None)),
            ("t2", TriggerKind::Manual),
        ]);
        let workflow_id = seed_workflow(pool, org_id, &graph).await;
        (org_id, workflow_id)
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_row_is_written_per_seed() {
        let pool = automation_pool().await;
        let (org_id, workflow_id) = scheduled_workflow(&pool, "sched-write").await;
        let at = now_storable() + Duration::hours(3);

        replace(
            &pool,
            org_id,
            workflow_id,
            &[seed("t1", at), seed("t2", at)],
        )
        .await;

        assert_eq!(
            rows(&pool, workflow_id).await,
            vec![("t1".to_string(), at, None), ("t2".to_string(), at, None)]
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn saving_again_replaces_the_rows_of_the_workflow_only() {
        let pool = automation_pool().await;
        let (org_id, workflow_id) = scheduled_workflow(&pool, "sched-replace").await;
        let (_, other) = scheduled_workflow(&pool, "sched-replace-other").await;
        let at = now_storable() + Duration::hours(3);
        replace(
            &pool,
            org_id,
            workflow_id,
            &[seed("t1", at), seed("t2", at)],
        )
        .await;
        replace(&pool, org_id, other, &[seed("t1", at)]).await;

        let later = at + Duration::days(1);
        replace(&pool, org_id, workflow_id, &[seed("t1", later)]).await;

        assert_eq!(
            rows(&pool, workflow_id).await,
            vec![("t1".to_string(), later, None)]
        );
        assert_eq!(rows(&pool, other).await.len(), 1);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn only_the_rows_that_are_due_are_claimed_with_what_the_worker_needs() {
        let pool = automation_pool().await;
        let (org_id, workflow_id) = scheduled_workflow(&pool, "sched-claim").await;
        set_organization_timezone(&pool, org_id, "America/New_York").await;
        let now = now_storable();
        let due_at = now - Duration::minutes(5);
        replace(
            &pool,
            org_id,
            workflow_id,
            &[seed("t1", due_at), seed("t2", now + Duration::minutes(5))],
        )
        .await;

        let claimed = claim_of(&pool, workflow_id, now).await;

        assert_eq!(claimed.len(), 1);
        let due = &claimed[0];
        assert_eq!(due.org_id, org_id);
        assert_eq!(due.trigger_id, "t1");
        assert_eq!(due.scheduled_for, due_at);
        assert_eq!(due.organization_timezone, "America/New_York");
        assert_eq!(due.trigger, daily("09:00", None));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_row_due_exactly_now_is_claimed() {
        let pool = automation_pool().await;
        let (org_id, workflow_id) = scheduled_workflow(&pool, "sched-boundary").await;
        let now = now_storable();
        replace(&pool, org_id, workflow_id, &[seed("t1", now)]).await;

        assert_eq!(claim_of(&pool, workflow_id, now).await.len(), 1);
        assert!(
            claim_of(&pool, workflow_id, now - Duration::seconds(1))
                .await
                .is_empty()
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_disabled_or_archived_workflow_is_never_claimed() {
        let pool = automation_pool().await;
        let (org_id, workflow_id) = scheduled_workflow(&pool, "sched-disabled").await;
        let now = now_storable();
        replace(
            &pool,
            org_id,
            workflow_id,
            &[seed("t1", now - Duration::days(1))],
        )
        .await;

        set_enabled(&pool, workflow_id, false).await;
        assert!(claim_of(&pool, workflow_id, now).await.is_empty());

        set_enabled(&pool, workflow_id, true).await;
        assert_eq!(claim_of(&pool, workflow_id, now).await.len(), 1);

        archive(&pool, workflow_id).await;
        assert!(claim_of(&pool, workflow_id, now).await.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_oldest_rows_come_first_and_the_batch_bounds_the_claim() {
        let pool = automation_pool().await;
        let (org_id, workflow_id) = scheduled_workflow(&pool, "sched-order").await;
        let now = now_storable();
        replace(
            &pool,
            org_id,
            workflow_id,
            &[
                seed("t1", now - Duration::minutes(1)),
                seed("t2", now - Duration::minutes(9)),
            ],
        )
        .await;

        let both = claim_of(&pool, workflow_id, now).await;
        let first = with_tx(&pool, async |tx| {
            PgScheduleRepository::new(&tx).claim_due(now, 1, 1000).await
        })
        .await
        .unwrap();

        assert_eq!(
            both.iter()
                .map(|due| due.trigger_id.as_str())
                .collect::<Vec<_>>(),
            vec!["t2", "t1"]
        );
        assert_eq!(first.len(), 1);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn advancing_moves_the_due_time_and_records_the_firing() {
        let pool = automation_pool().await;
        let (org_id, workflow_id) = scheduled_workflow(&pool, "sched-advance").await;
        let now = now_storable();
        replace(
            &pool,
            org_id,
            workflow_id,
            &[
                seed("t1", now - Duration::minutes(1)),
                seed("t2", now - Duration::minutes(1)),
            ],
        )
        .await;
        let next = now + Duration::days(1);

        with_tx(&pool, async |tx| {
            let mut repo = PgScheduleRepository::new(&tx);
            repo.advance(workflow_id, "t1", Some(now), next).await?;
            repo.advance(workflow_id, "t2", None, next).await
        })
        .await
        .unwrap();

        assert_eq!(
            rows(&pool, workflow_id).await,
            vec![
                ("t1".to_string(), next, Some(now)),
                ("t2".to_string(), next, None)
            ]
        );
        assert!(claim_of(&pool, workflow_id, now).await.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_row_another_transaction_has_claimed_is_skipped_not_waited_on() {
        let pool = automation_pool().await;
        let (org_id, workflow_id) = scheduled_workflow(&pool, "sched-skip").await;
        let now = now_storable();
        replace(
            &pool,
            org_id,
            workflow_id,
            &[seed("t1", now - Duration::minutes(1))],
        )
        .await;

        let (claimed_tx, claimed_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = tokio::sync::oneshot::channel::<()>();
        let holder = tokio::spawn({
            let pool = pool.clone();
            async move {
                with_tx(&pool, async move |tx| {
                    let held = PgScheduleRepository::new(&tx)
                        .claim_due(now, 1000, 1000)
                        .await?;
                    claimed_tx.send(held).ok();
                    release_rx.await.ok();
                    Ok::<_, CoreError>(())
                })
                .await
            }
        });
        let held = claimed_rx.await.unwrap();
        let concurrent = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            claim_of(&pool, workflow_id, now),
        )
        .await
        .expect("a claim must skip a locked row, not wait for it");
        release_tx.send(()).ok();
        holder.await.unwrap().unwrap();

        assert!(held.iter().any(|due| due.workflow_id == workflow_id));
        assert!(concurrent.is_empty());
        assert_eq!(claim_of(&pool, workflow_id, now).await.len(), 1);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_the_workflow_removes_its_rows() {
        let pool = automation_pool().await;
        let (org_id, workflow_id) = scheduled_workflow(&pool, "sched-delete").await;
        replace(&pool, org_id, workflow_id, &[seed("t1", now_storable())]).await;
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

    async fn seed_many(
        pool: &PgPool,
        org_id: OrganizationId,
        count: usize,
        due: DateTime<Utc>,
    ) -> Vec<Uuid> {
        let mut workflows = Vec::new();
        for _ in 0..count {
            let graph = graph_of(vec![("t1", daily("09:00", None))]);
            let workflow_id = seed_workflow(pool, org_id, &graph).await;
            replace(pool, org_id, workflow_id, &[seed("t1", due)]).await;
            workflows.push(workflow_id);
        }
        workflows
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn one_organization_cannot_take_more_than_its_share_of_a_pass() {
        let pool = automation_pool().await;
        let noisy = seed_organization(&pool, "sched-noisy").await;
        let quiet = seed_organization(&pool, "sched-quiet").await;
        let now = now_storable();
        let due = now - Duration::minutes(5);
        let noisy_workflows = seed_many(&pool, noisy, 15, due).await;
        let quiet_workflows = seed_many(&pool, quiet, 1, due + Duration::minutes(1)).await;

        let claimed = with_tx(&pool, async |tx| {
            PgScheduleRepository::new(&tx).claim_due(now, 100, 10).await
        })
        .await
        .unwrap();

        let of = |workflows: &[Uuid]| {
            claimed
                .iter()
                .filter(|due| workflows.contains(&due.workflow_id))
                .count()
        };
        assert_eq!(of(&noisy_workflows), 10);
        assert_eq!(of(&quiet_workflows), 1);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_oldest_rows_of_an_organization_are_the_ones_its_share_covers() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "sched-share-order").await;
        let now = now_storable();
        let mut workflows = Vec::new();
        for minutes in (1..=12).rev() {
            let id = seed_many(&pool, org_id, 1, now - Duration::minutes(minutes)).await[0];
            workflows.push((minutes, id));
        }

        let claimed = with_tx(&pool, async |tx| {
            PgScheduleRepository::new(&tx).claim_due(now, 100, 10).await
        })
        .await
        .unwrap();

        let mut ages: Vec<i64> = workflows
            .iter()
            .filter(|(_, id)| claimed.iter().any(|due| due.workflow_id == *id))
            .map(|(minutes, _)| *minutes)
            .collect();
        ages.sort_unstable();
        assert_eq!(ages, (3..=12).collect::<Vec<i64>>());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_workflow_being_disabled_is_not_claimed_from_under_the_change() {
        let pool = automation_pool().await;
        let (org_id, workflow_id) = scheduled_workflow(&pool, "sched-lock-workflow").await;
        let now = now_storable();
        replace(
            &pool,
            org_id,
            workflow_id,
            &[seed("t1", now - Duration::minutes(1))],
        )
        .await;

        let (changing_tx, changing_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = tokio::sync::oneshot::channel::<()>();
        let changer = tokio::spawn({
            let pool = pool.clone();
            async move {
                let mut tx = pool.begin().await.unwrap();
                sqlx::query("UPDATE automation.workflow SET enabled = false WHERE id = $1")
                    .bind(workflow_id)
                    .execute(&mut *tx)
                    .await
                    .unwrap();
                changing_tx.send(()).ok();
                release_rx.await.ok();
                tx.commit().await.unwrap();
            }
        });
        changing_rx.await.unwrap();
        let during = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            claim_of(&pool, workflow_id, now),
        )
        .await
        .expect("a claim must skip a workflow that is being changed, not wait for it");
        release_tx.send(()).ok();
        changer.await.unwrap();
        let after = claim_of(&pool, workflow_id, now).await;

        assert!(
            during.is_empty(),
            "claimed a workflow being disabled: {during:?}"
        );
        assert!(after.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn saving_again_keeps_when_a_schedule_last_ran() {
        let pool = automation_pool().await;
        let (org_id, workflow_id) = scheduled_workflow(&pool, "sched-keep-last").await;
        let ran = now_storable() - Duration::hours(2);
        let first = now_storable() + Duration::hours(3);
        replace(
            &pool,
            org_id,
            workflow_id,
            &[seed("t1", first), seed("t2", first)],
        )
        .await;
        with_tx(&pool, async |tx| {
            let mut repo = PgScheduleRepository::new(&tx);
            repo.advance(workflow_id, "t1", Some(ran), first).await?;
            repo.advance(workflow_id, "t2", Some(ran), first).await
        })
        .await
        .unwrap();

        let later = first + Duration::days(1);
        replace(&pool, org_id, workflow_id, &[seed("t1", later)]).await;

        assert_eq!(
            rows(&pool, workflow_id).await,
            vec![("t1".to_string(), later, Some(ran))]
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_schedules_held_by_other_workflows_of_the_organization_are_counted() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "sched-count").await;
        let other_org = seed_organization(&pool, "sched-count-other").await;
        let now = now_storable() + Duration::hours(1);
        let first = seed_many(&pool, org_id, 1, now).await[0];
        let second = seed_many(&pool, org_id, 1, now).await[0];
        seed_many(&pool, other_org, 3, now).await;

        let count = |workflow_id: Uuid| {
            let pool = pool.clone();
            async move {
                with_tx(&pool, async |tx| {
                    PgScheduleRepository::new(&tx)
                        .count_other_schedules(org_id, workflow_id)
                        .await
                })
                .await
                .unwrap()
            }
        };

        assert_eq!(count(first).await, 1);
        assert_eq!(count(second).await, 1);
        assert_eq!(count(Uuid::now_v7()).await, 2);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_dropped_attempt_undoes_what_it_did_even_after_a_database_error() {
        let pool = automation_pool().await;
        let (org_id, workflow_id) = scheduled_workflow(&pool, "sched-attempt").await;
        let due = now_storable() + Duration::hours(1);
        replace(
            &pool,
            org_id,
            workflow_id,
            &[seed("t1", due), seed("t2", due)],
        )
        .await;
        let moved = due + Duration::days(1);

        with_tx(&pool, async |tx| {
            let mut repo = PgScheduleRepository::new(&tx);
            repo.open_attempt().await?;
            repo.advance(workflow_id, "t1", None, moved).await?;
            {
                let mut inner = tx.lock().await;
                let broken = sqlx::query("SELECT 1 FROM automation.no_such_table")
                    .execute(&mut ***inner)
                    .await;
                assert!(broken.is_err());
            }
            repo.drop_attempt().await?;
            repo.open_attempt().await?;
            repo.advance(workflow_id, "t2", None, moved).await?;
            repo.keep_attempt().await
        })
        .await
        .unwrap();

        assert_eq!(
            rows(&pool, workflow_id).await,
            vec![
                ("t1".to_string(), due, None),
                ("t2".to_string(), moved, None)
            ]
        );
    }
}
