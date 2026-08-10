use std::time::Duration;

use chrono::Utc;
use tracing::{error, info, warn};

use crate::{
    application::{MestierUseCase, automation::run::EnginePassOutcome},
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
/// loudly.
pub async fn run_automation_worker(
    usecase: MestierUseCase,
    worker: String,
    schedule: WorkerSchedule,
    private_network: PrivateNetworkAccess,
) {
    info!(%worker, "automation worker started");
    let mut ticker = tokio::time::interval(schedule.interval);
    // Built once, not per tick: its only state is a clone of `usecase`, the
    // same handle the run-engine pass already carries.
    let connectors =
        ConnectorRegistry::with_private_network_access(usecase.clone(), private_network);

    loop {
        ticker.tick().await;

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

    #[test]
    fn an_absurd_claim_timeout_falls_back_rather_than_panicking() {
        assert_eq!(
            chrono_from(Duration::from_secs(u64::MAX)),
            chrono::Duration::minutes(5)
        );
    }
}
