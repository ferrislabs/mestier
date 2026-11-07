use std::{future::Future, sync::Arc, time::Duration};

use chrono::Utc;
use tracing::{error, info, warn};

use crate::{
    application::{
        MestierUseCase,
        automation::{run::EnginePassOutcome, trigger::ScheduleOutcome},
    },
    infrastructure::automation::{
        connectors::ConnectorRegistry, webhook::address_policy::PrivateNetworkAccess,
    },
};

/// How the background loop paces itself.
///
/// Polling rather than `LISTEN`/`NOTIFY` alone: a retry becomes due with no
/// event to announce it, so the loop has to wake on its own regardless. The
/// notification the emitter fires is a latency improvement to add on top, not
/// a replacement.
#[derive(Debug, Clone, Copy)]
pub struct WorkerSchedule {
    pub interval: Duration,
    /// Runs claimed per pass, across all organizations.
    pub batch: i64,
    /// Ceiling on how much of a batch one organization may take, so a noisy
    /// tenant cannot starve the others.
    pub per_org: i64,
    /// A claim older than this belonged to a worker that died.
    pub claim_timeout: Duration,
    /// How much of one run's graph a single slice may work through before
    /// handing the run back — `pending`, not lost — for a later pass.
    /// Bounds how long a claimed run can occupy the worker: without this a
    /// 500-element `flow.loop` would monopolize a slot for however long the
    /// whole loop takes, the same way a single delivery cannot.
    pub max_steps_per_slice: usize,
    pub max_slice_duration: Duration,
}

impl Default for WorkerSchedule {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(5),
            batch: 100,
            per_org: 20,
            claim_timeout: Duration::from_secs(300),
            max_steps_per_slice: 50,
            max_slice_duration: Duration::from_secs(20),
        }
    }
}

/// Runs the automation loop until the process ends.
///
/// Nothing here propagates an error: a failing pass must not take the process
/// down, because the events and runs it could not process are still in the
/// database and the next pass will find them. What it must do is say so
/// loudly. A pass runs in a task of its own for the same reason: a panic in
/// one pass is read back as a `JoinError`, logged, and the next tick starts a
/// fresh pass instead of the loop dying with it.
pub async fn run_automation_worker(
    usecase: MestierUseCase,
    worker: String,
    schedule: WorkerSchedule,
    private_network: PrivateNetworkAccess,
) {
    info!(%worker, "automation worker started");
    // Built once, not per tick: its only state is a clone of `usecase`, the
    // same handle the run-engine pass already carries.
    let connectors = Arc::new(ConnectorRegistry::with_private_network_access(
        usecase.clone(),
        private_network,
    ));

    run_supervised(schedule.interval, move || {
        pass(
            usecase.clone(),
            Arc::clone(&connectors),
            worker.clone(),
            schedule,
        )
    })
    .await;
}

async fn run_supervised<F, Fut>(interval: Duration, mut next_pass: F)
where
    F: FnMut() -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    let mut ticker = tokio::time::interval(interval);
    loop {
        ticker.tick().await;
        supervised(next_pass()).await;
    }
}

async fn supervised<Fut>(pass: Fut) -> bool
where
    Fut: Future<Output = ()> + Send + 'static,
{
    match tokio::spawn(pass).await {
        Ok(()) => true,
        Err(error) if error.is_panic() => {
            let payload = error.into_panic();
            let message = payload
                .downcast_ref::<&str>()
                .map(|message| (*message).to_owned())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "no message".to_owned());
            error!(%message, "an automation pass panicked; the next pass starts on schedule");
            false
        }
        Err(error) => {
            error!(%error, "an automation pass was cancelled; the next pass starts on schedule");
            false
        }
    }
}

async fn pass(
    usecase: MestierUseCase,
    connectors: Arc<ConnectorRegistry>,
    worker: String,
    schedule: WorkerSchedule,
) {
    match usecase
        .recover_lost_runs(Utc::now() - chrono_from(schedule.claim_timeout))
        .await
    {
        Ok(released) if released > 0 => {
            warn!(released, "recovered runs from a worker that was lost");
        }
        Ok(_) => {}
        Err(error) => error!(%error, "recovering lost runs failed"),
    }

    if let Err(error) = usecase.dispatch_pending_events(schedule.batch).await {
        error!(%error, "fanning events out failed");
    }

    // Same shape as the event fan-out just above: turns state that
    // needs attention (a recurrence whose horizon is getting close to
    // today) into `automation.run` rows, on the same tick, through the
    // same queue `run_engine_pass` already claims from below — never a
    // second scheduler.
    match usecase.ensure_recurrence_horizon_runs().await {
        Ok(created) if created > 0 => {
            info!(created, "queued recurrence horizon extension runs");
        }
        Ok(_) => {}
        Err(error) => error!(%error, "scheduling recurrence horizon extension runs failed"),
    }

    match usecase.fire_due_schedules(Utc::now(), schedule.batch).await {
        Ok(ScheduleOutcome {
            fired,
            unreadable,
            failed,
        }) if fired > 0 || unreadable > 0 || failed > 0 => {
            info!(fired, unreadable, failed, "started scheduled runs");
        }
        Ok(_) => {}
        Err(error) => error!(%error, "starting scheduled runs failed"),
    }

    match usecase
        .run_engine_pass(&connectors, &worker, schedule)
        .await
    {
        Ok(EnginePassOutcome {
            claimed,
            succeeded,
            rescheduled,
            failed,
        }) if claimed > 0 => {
            info!(claimed, succeeded, rescheduled, failed, "run engine pass");
        }
        Ok(_) => {}
        Err(error) => error!(%error, "the run engine pass failed"),
    }
}

/// `chrono` and `std` disagree on duration types, and the conversion can only
/// fail for durations no schedule will ever hold.
fn chrono_from(duration: Duration) -> chrono::Duration {
    chrono::Duration::from_std(duration).unwrap_or_else(|_| chrono::Duration::minutes(5))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_schedule_is_conservative() {
        let schedule = WorkerSchedule::default();

        assert!(
            schedule.per_org < schedule.batch,
            "one tenant cannot take it all"
        );
        assert!(
            schedule.claim_timeout > schedule.interval,
            "a claim must outlive a pass, or a working delivery is stolen from itself"
        );
    }

    #[tokio::test]
    async fn a_pass_that_panics_is_reported_as_failed_and_does_not_stop_the_next_one() {
        let ran = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let survived = supervised(async { panic!("a pass blew up") }).await;
        let counter = Arc::clone(&ran);
        let next = supervised(async move {
            counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        })
        .await;

        assert!(!survived);
        assert!(next);
        assert_eq!(ran.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn the_loop_keeps_starting_passes_after_one_panicked() {
        let started = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter = Arc::clone(&started);

        let _ = tokio::time::timeout(
            Duration::from_millis(300),
            run_supervised(Duration::from_millis(10), move || {
                let call = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                async move {
                    if call == 0 {
                        panic!("the first pass blows up");
                    }
                }
            }),
        )
        .await;

        assert!(
            started.load(std::sync::atomic::Ordering::SeqCst) >= 3,
            "passes kept coming after the panic"
        );
    }

    #[test]
    fn an_absurd_claim_timeout_falls_back_rather_than_panicking() {
        assert_eq!(
            chrono_from(Duration::from_secs(u64::MAX)),
            chrono::Duration::minutes(5)
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_worker_loop_starts_the_schedules_that_are_due() {
        use chrono::Utc;
        use common::OrganizationId;
        use uuid::Uuid;

        use crate::application::automation::trigger_fixtures::{
            daily, graph_of, seed_organization,
        };
        use crate::application::default_authorizer;
        use crate::application::test_support::automation_pool;
        use crate::domain::automation::workflow::{
            CreateWorkflowCommand, SaveWorkflowVersionCommand,
        };
        use crate::infrastructure::realtime::EventHub;

        let pool = automation_pool().await;
        let org_id: OrganizationId = seed_organization(&pool, "worker-schedule").await;
        let usecase = MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new());
        let workflow_id: Uuid = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Worker schedule".to_string(),
                description: None,
            })
            .await
            .unwrap()
            .id;
        usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id,
                graph: graph_of(vec![("t1", daily("09:00", Some("UTC")))]),
                layout: None,
                created_by: None,
            })
            .await
            .unwrap();
        let overdue = Utc::now() - chrono::Duration::seconds(1);
        sqlx::query!(
            "UPDATE automation.schedule_state SET next_run_at = $2 WHERE workflow_id = $1",
            workflow_id,
            overdue,
        )
        .execute(&pool)
        .await
        .unwrap();

        let schedule = WorkerSchedule {
            interval: Duration::from_millis(50),
            ..WorkerSchedule::default()
        };
        let worker = tokio::spawn(run_automation_worker(
            usecase,
            "test-worker".to_owned(),
            schedule,
            PrivateNetworkAccess::Denied,
        ));
        let mut started = 0;
        for _ in 0..100 {
            started = sqlx::query_scalar!(
                r#"SELECT count(*) AS "count!" FROM automation.run WHERE workflow_id = $1"#,
                workflow_id,
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            if started > 0 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        tokio::time::sleep(Duration::from_millis(400)).await;
        let after = sqlx::query_scalar!(
            r#"SELECT count(*) AS "count!" FROM automation.run WHERE workflow_id = $1"#,
            workflow_id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        worker.abort();

        assert_eq!(started, 1);
        assert_eq!(
            after, 1,
            "a due schedule fires once, whatever the ticks that follow"
        );
    }
}
